//! Public-API contract of `file`: whole and streaming reads, atomic writes, directories and confined paths.

#[cfg(test)]
mod tests {

    use std::path::{Path as StdPath, PathBuf};
    use engine::file::{Dir, Entry, File, Kind, Path};
    use engine::memory::{Space as _, Stack};
    use engine::error::{Code, Result};
    use std::env::temp_dir;
    use std::process::id;
    use engine::thread::Runtime;

    fn scratch ( name: &str ) -> PathBuf {

        temp_dir().join(format!("engine-file-{}-{name}", id()))

    }

    async fn roundtrip ( dir: &StdPath ) -> Result<( Vec<u8>, usize, Vec<u8>, bool, u64 )> {

        Dir::ensure(dir).await?;

        let path = dir.join("note.txt");

        File::write(&path, b"first").await?;
        File::write(&path, b"second version").await?;

        let whole = File::read(&path).await?.to_vec();
        let mut small = Stack::<6>::new();
        let came = File::read_into(&path, &mut small).await?;

        File::append(&path, b"!").await?;

        let appended = File::read(&path).await?.to_vec();
        let leftovers = Dir::list(dir).await?.iter().any(|entry| Path::extension(entry.path()).as_deref() == Some("tmp"));
        let len = File::info(&path).await?.len();

        Dir::remove(dir).await?;

        Ok(( whole, came, appended, leftovers, len ))

    }

    #[test]
    fn writes_land_whole_and_reads_come_back () {

        let outcome = Runtime::block_on(roundtrip(&scratch("roundtrip"))).ok().and_then(Result::ok);

        assert_eq!(outcome, Some(( b"second version".to_vec(), 6, b"second version!".to_vec(), false, 15 )));

    }

    async fn abandoned ( dir: &StdPath ) -> Result<( bool, usize )> {

        Dir::ensure(dir).await?;

        let target = dir.join("never.txt");
        let mut writer = File::create(&target).await?;

        writer.write(b"half").await?;
        drop(writer);

        let exists = File::exists(&target).await;
        let count  = Dir::list(dir).await?.len();

        Dir::remove(dir).await?;

        Ok(( exists, count ))

    }

    #[test]
    fn an_uncommitted_writer_leaves_nothing_behind () {

        assert_eq!(Runtime::block_on(abandoned(&scratch("abandoned"))).ok().and_then(Result::ok), Some(( false, 0 )));

    }

    async fn inspected ( dir: &StdPath ) -> Result<( Kind, Kind, bool, bool, Option<Code> )> {

        Dir::ensure(dir.join("sub")).await?;
        File::write(dir.join("a.txt"), b"abc").await?;

        let file = File::info(dir.join("a.txt")).await?;
        let sub  = File::peek(dir.join("sub")).await?;
        let gone = File::info(dir.join("missing")).await.err().map(|failure| failure.code());

        File::remove(dir.join("missing")).await?;
        File::remove(dir.join("a.txt")).await?;

        let after = File::exists(dir.join("a.txt")).await;
        let isdir = Dir::exists(dir.join("sub")).await;

        Dir::remove(dir).await?;

        Ok(( file.kind(), sub.kind(), after, isdir, gone ))

    }

    #[test]
    fn info_peek_exists_and_remove_agree () {

        assert_eq!(Runtime::block_on(inspected(&scratch("inspect"))).ok().and_then(Result::ok), Some(( Kind::File, Kind::Dir, false, true, Some(Code::NotFound) )));

    }

    async fn walked ( dir: &StdPath ) -> Result<( Vec<String>, bool, usize, usize )> {

        Dir::ensure(dir.join("b").join("deep")).await?;
        File::write(dir.join("c.txt"), b"c").await?;
        File::write(dir.join("a.txt"), b"a").await?;
        File::write(dir.join("b").join("deep").join("d.txt"), b"d").await?;

        let entries = Dir::list(dir).await?;
        let listed: Vec<String> = entries.iter().map(|entry| entry.name().to_owned()).collect();
        let first   = entries.into_iter().next().map(Entry::into_path) == Some(dir.join("a.txt"));
        let shallow = Dir::walk(dir, 0).await?.len();
        let deep    = Dir::walk(dir, 5).await?.len();

        Dir::remove(dir).await?;
        Dir::remove(dir).await?;

        Ok(( listed, first, shallow, deep ))

    }

    #[test]
    fn directories_list_sorted_and_walk_to_a_depth () {

        assert_eq!(Runtime::block_on(walked(&scratch("walk"))).ok().and_then(Result::ok), Some(( vec!["a.txt".to_owned(), "b".to_owned(), "c.txt".to_owned()], true, 3, 5 )));

    }

    async fn streamed ( dir: &StdPath ) -> Result<( u64, usize, Vec<u8>, Vec<u8> )> {

        Dir::ensure(dir).await?;

        let path = dir.join("stream.bin");

        File::write(&path, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).await?;

        let mut reader = File::open(&path).await?;
        let mut head = Stack::<4>::new();
        let came = reader.read(&mut head).await?;
        let rest = reader.rest().await?.to_vec();

        reader.seek(8).await?;

        let tail = reader.rest().await?.to_vec();
        let len  = reader.len();

        File::copy(&path, dir.join("copy.bin")).await?;
        File::rename(dir.join("copy.bin"), dir.join("moved.bin")).await?;

        let moved = File::read(dir.join("moved.bin")).await?.len();

        Dir::remove(dir).await?;

        Ok(( len, came + moved, [head.as_bytes(), &rest].concat(), tail ))

    }

    #[test]
    fn readers_stream_seek_and_files_copy_and_move () {

        assert_eq!(Runtime::block_on(streamed(&scratch("stream"))).ok().and_then(Result::ok), Some(( 10, 14, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10], vec![9, 10] )));

    }

    #[test]
    fn paths_stay_inside_their_root () {

        let root = StdPath::new("/srv/site");

        assert_eq!(Path::confine(root, StdPath::new("a/b.txt")).ok(), Some(root.join("a").join("b.txt")));
        assert_eq!(Path::confine(root, StdPath::new("a/../b.txt")).ok(), Some(root.join("b.txt")));
        assert_eq!(Path::confine(root, StdPath::new("./a/./b")).ok(), Some(root.join("a").join("b")));
        assert_eq!(Path::confine(root, StdPath::new("")).ok(), Some(root.to_path_buf()));
        assert_eq!(Path::confine(root, StdPath::new("../etc/passwd")).err().map(|failure| ( failure.code(), failure.key() )), Some(( Code::Forbidden, "escapes_root" )));
        assert_eq!(Path::confine(root, StdPath::new("a/../../x")).err().map(|failure| failure.key()), Some("escapes_root"));
        assert_eq!(Path::confine(root, StdPath::new("/abs")).err().map(|failure| failure.key()), Some("escapes_root"));

    }

    #[test]
    fn paths_normalize_and_name_their_parts () {

        assert_eq!(Path::normalize(StdPath::new("a/./b/../c")), PathBuf::from("a/c"));
        assert_eq!(Path::normalize(StdPath::new("../a")), PathBuf::from("../a"));
        assert_eq!(Path::normalize(StdPath::new("/a/../../b")), PathBuf::from("/../b"));
        assert_eq!(Path::extension(StdPath::new("x/Y.TXT")).as_deref(), Some("txt"));
        assert_eq!(Path::extension(StdPath::new("x/y")), None);
        assert_eq!(Path::stem(StdPath::new("x/y.tar.gz")), Some("y.tar"));
        assert!(Path::is_hidden(StdPath::new("x/.env")));
        assert!(!Path::is_hidden(StdPath::new("x/env")));

    }

}
