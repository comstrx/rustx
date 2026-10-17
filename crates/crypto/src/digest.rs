use crate::Digest;

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

impl Digest {

    /// The SHA-1 of the bytes.
    #[inline]
    #[must_use]
    pub fn sha1 ( data: &[u8] ) -> [u8; 20] {

        let mut state: [u32; 5] = [0x6745_2301, 0xEFCD_AB89, 0x98BA_DCFE, 0x1032_5476, 0xC3D2_E1F0];
        let mut padded = Vec::with_capacity(data.len() + 72);

        padded.extend_from_slice(data);
        padded.push(0x80);

        while !(padded.len() + 8).is_multiple_of(64) { padded.push(0); }

        padded.extend_from_slice(&(u64::try_from(data.len()).unwrap_or(u64::MAX).wrapping_mul(8)).to_be_bytes());

        for block in padded.as_chunks::<64>().0 { Self::round(&mut state, block); }

        let mut out = [0u8; 20];

        for ( slot, word ) in out.as_chunks_mut::<4>().0.iter_mut().zip(state) { *slot = word.to_be_bytes(); }

        out

    }

    /// The standard Base64 of the bytes, padded.
    #[inline]
    #[must_use]
    pub fn base64 ( data: &[u8] ) -> String {

        let ( chunks, tail ) = data.as_chunks::<3>();
        let mut out = String::with_capacity(data.len().div_ceil(3) * 4);

        for chunk in chunks { Self::quad(&mut out, u32::from_be_bytes([0, chunk[0], chunk[1], chunk[2]]), 4); }

        match *tail {
            [first]         => Self::quad(&mut out, u32::from_be_bytes([0, first, 0, 0]), 2),
            [first, second] => Self::quad(&mut out, u32::from_be_bytes([0, first, second, 0]), 3),
            _               => {}
        }

        out

    }

    fn quad ( out: &mut String, bits: u32, keep: usize ) {

        for index in 0..4 { out.push(char::from(Self::glyph(bits, index, keep))); }

    }

    fn glyph ( bits: u32, index: usize, keep: usize ) -> u8 {

        if index >= keep { return b'='; }

        let shift = 18 - 6 * u32::try_from(index).unwrap_or(3);

        ALPHABET.get(usize::try_from((bits >> shift) & 63).unwrap_or_default()).copied().unwrap_or(b'=')

    }

    fn round ( state: &mut [u32; 5], block: &[u8; 64] ) {

        let mut words = [0u32; 80];

        for ( word, chunk ) in words.iter_mut().zip(block.as_chunks::<4>().0) { *word = u32::from_be_bytes(*chunk); }

        for index in 16..80 { Self::expand(&mut words, index); }

        let [mut one, mut two, mut three, mut four, mut five] = *state;

        for ( index, word ) in words.iter().enumerate() {

            let ( mix, constant ) = Self::stage(index, two, three, four);
            let temp = one.rotate_left(5).wrapping_add(mix).wrapping_add(five).wrapping_add(constant).wrapping_add(*word);

            five = four;
            four = three;
            three = two.rotate_left(30);
            two = one;
            one = temp;

        }

        for ( slot, value ) in state.iter_mut().zip([one, two, three, four, five]) { *slot = slot.wrapping_add(value); }

    }

    fn expand ( words: &mut [u32; 80], index: usize ) {

        let back = |steps: usize| words.get(index - steps).copied().unwrap_or_default();
        let mixed = (back(3) ^ back(8) ^ back(14) ^ back(16)).rotate_left(1);

        if let Some(slot) = words.get_mut(index) { *slot = mixed; }

    }

    const fn stage ( index: usize, two: u32, three: u32, four: u32 ) -> ( u32, u32 ) {

        match index {
            0..=19  => ( (two & three) | (!two & four), 0x5A82_7999 ),
            20..=39 => ( two ^ three ^ four, 0x6ED9_EBA1 ),
            40..=59 => ( (two & three) | (two & four) | (three & four), 0x8F1B_BCDC ),
            _       => ( two ^ three ^ four, 0xCA62_C1D6 ),
        }

    }

}
