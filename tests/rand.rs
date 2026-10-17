//! Public-API contract of `rand`: uniform numbers, reals, strings, identifiers, picks, shuffles, dates and UUIDs.

#[cfg(test)]
mod tests {

    use std::time::{SystemTime, UNIX_EPOCH};
    use engine::error::{Code, Result};
    use engine::rand::{Random, Uuid};
    use std::collections::HashSet;
    use core::iter::repeat_with;
    use core::time::Duration;
    use std::thread::sleep;

    #[test]
    #[cfg_attr(miri, ignore = "thousands of rounds; the native suite covers it")]
    fn numbers_stay_inside_their_range_and_cover_it () {

        let drawn = repeat_with(|| Random::number(-3..=3)).take(4000).collect::<Result<Vec<i64>>>().unwrap_or_default();
        let seen: HashSet<i64> = drawn.iter().copied().collect();
        let ( low, high ) = ( 2, 1 );

        assert_eq!(drawn.len(), 4000);
        assert!(drawn.iter().all(|value| (-3..=3).contains(value)));
        assert_eq!(seen.len(), 7);
        assert_eq!(Random::number(5..=5).ok(), Some(5));
        assert!(Random::number(i64::MIN..=i64::MAX).ok().is_some());
        assert_eq!(Random::number(low..=high).err().map(|failure| ( failure.code(), failure.key() )), Some(( Code::Invalid, "empty_range" )));
        assert_eq!(Random::below(0).err().map(|failure| failure.key()), Some("empty_range"));
        assert_eq!(Random::index(0).err().map(|failure| failure.key()), Some("empty_range"));

    }

    #[test]
    #[cfg_attr(miri, ignore = "thousands of rounds; the native suite covers it")]
    fn draws_are_uniform_within_a_tight_band () {

        let drawn  = repeat_with(|| Random::below(6)).take(60_000).collect::<Result<Vec<u64>>>().unwrap_or_default();
        let counts = ( 0..6u64 ).map(|face| drawn.iter().filter(|&&value| value == face).count()).collect::<Vec<usize>>();

        assert_eq!(drawn.len(), 60_000);
        assert!(counts.iter().all(|count| (9_400..=10_600).contains(count)), "{counts:?}");

    }

    #[test]
    #[cfg_attr(miri, ignore = "thousands of rounds; the native suite covers it")]
    fn reals_and_chances_keep_their_bounds () {

        let reals = repeat_with(Random::real).take(10_000).collect::<Result<Vec<f64>>>().unwrap_or_default();
        let mean  = reals.iter().sum::<f64>() / 10_000.0;

        assert_eq!(reals.len(), 10_000);
        assert!(reals.iter().all(|real| (0.0..1.0).contains(real)));
        assert!((0.47..0.53).contains(&mean), "{mean}");
        assert_eq!(Random::chance(0.0).ok(), Some(false));
        assert_eq!(Random::chance(1.0).ok(), Some(true));
        assert_eq!(Random::chance(f64::NAN).ok(), Some(false));

    }

    #[test]
    #[cfg_attr(miri, ignore = "thousands of rounds; the native suite covers it")]
    fn strings_and_identifiers_use_only_their_alphabet () {

        let string = Random::string(64).unwrap_or_default();
        let text   = Random::text(32, "ab").unwrap_or_default();
        let ids    = repeat_with(Random::id).take(1000).collect::<Result<HashSet<String>>>().unwrap_or_default();

        assert!(string.len() == 64 && string.chars().all(|symbol| symbol.is_ascii_alphanumeric()));
        assert!(text.len() == 32 && text.chars().all(|symbol| symbol == 'a' || symbol == 'b'));
        assert_eq!(ids.len(), 1000);
        assert!(ids.iter().all(|id| id.len() == Random::ID_LEN && id.chars().all(|symbol| Random::URL_SAFE.contains(symbol))));
        assert_eq!(Random::text(3, "").err().map(|failure| failure.key()), Some("empty_range"));
        assert_eq!(Random::text(0, "").ok(), Some(String::new()));
        assert_eq!(Random::text(4, "\u{e9}").ok(), Some("\u{e9}".repeat(4)));
        assert_eq!(Random::bytes(48).map(|bytes| bytes.len()).ok(), Some(48));
        assert_eq!(Random::bytes(4096).map(|bytes| bytes.len()).ok(), Some(4096));

    }

    #[test]
    fn picks_and_shuffles_keep_every_item () {

        let items = [1, 2, 3, 4, 5, 6, 7, 8];
        let picked = Random::pick(&items).ok().flatten().copied();
        let orders = repeat_with(|| { let mut order = items; Random::shuffle(&mut order).map(|()| order) }).take(64).collect::<Result<HashSet<[i32; 8]>>>().unwrap_or_default();

        assert!(picked.is_some_and(|item| items.contains(&item)));
        assert_eq!(Random::pick::<u8>(&[]).ok(), Some(None));
        assert!(orders.len() > 48, "{}", orders.len());
        assert!(orders.iter().all(|order| { let mut sorted = *order; sorted.sort_unstable(); sorted == items }));

    }

    #[test]
    #[cfg_attr(miri, ignore = "thousands of rounds; the native suite covers it")]
    fn dates_fall_between_their_bounds_in_either_order () {

        let from  = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let until = from + Duration::from_hours(24);
        let dates = repeat_with(|| Random::date(until, from)).take(1000).collect::<Result<Vec<SystemTime>>>().unwrap_or_default();

        assert_eq!(dates.len(), 1000);
        assert!(dates.iter().all(|date| (from..=until).contains(date)));
        assert_eq!(Random::date(from, from).ok(), Some(from));

    }

    #[test]
    #[cfg_attr(miri, ignore = "thousands of rounds; the native suite covers it")]
    fn uuids_carry_their_version_and_variant () {

        let random  = Uuid::random().unwrap_or_default();
        let ordered = Uuid::ordered().unwrap_or_default();
        let many    = repeat_with(Uuid::random).take(2000).collect::<Result<HashSet<Uuid>>>().unwrap_or_default();

        assert_eq!(( random.version(), ordered.version() ), ( 4, 7 ));
        assert!(random.as_bytes()[8] & 0xC0 == 0x80 && ordered.as_bytes()[8] & 0xC0 == 0x80);
        assert_ne!(random, Uuid::NIL);
        assert_eq!(many.len(), 2000);

    }

    #[test]
    fn ordered_uuids_sort_by_their_time () {

        let first = Uuid::ordered().unwrap_or_default();

        sleep(Duration::from_millis(3));

        let second = Uuid::ordered().unwrap_or_default();
        let [ b00, b01, b02, b03, b04, b05, .. ] = *second.as_bytes();
        let stamp  = u64::from_be_bytes([0, 0, b00, b01, b02, b03, b04, b05]);
        let now    = SystemTime::now().duration_since(UNIX_EPOCH).map(|elapsed| elapsed.as_millis()).unwrap_or_default();

        assert!(first < second);
        assert!(now.abs_diff(u128::from(stamp)) < 10_000);

    }

    #[test]
    fn uuids_print_and_parse_back () {

        let id   = Uuid::random().unwrap_or_default();
        let text = id.to_string();

        assert_eq!(text.len(), 36);
        assert_eq!(Uuid::parse(&text).ok(), Some(id));
        assert_eq!(Uuid::parse(&text.to_uppercase()).ok(), Some(id));
        assert_eq!(text.parse::<Uuid>().ok(), Some(id));
        assert_eq!(format!("{id:?}"), text);
        assert_eq!(Uuid::from_bytes(*id.as_bytes()), id);
        assert_eq!(Uuid::NIL.to_string(), "00000000-0000-0000-0000-000000000000");
        assert_eq!(Uuid::parse("67e55044-10b1-426f-9247-bb680e5fe0c8").map(|parsed| parsed.version()).ok(), Some(4));

        for bad in ["", "67e55044-10b1-426f-9247-bb680e5fe0c", "67e5504410b1-426f-9247-bb680e5fe0c8x", "67e55044-10b1-426f-9247-bb680e5fe0cg", "{67e55044-10b1-426f-9247-bb680e5fe0c8}"] {

            assert_eq!(Uuid::parse(bad).err().map(|failure| failure.key()), Some("not_uuid"), "{bad}");

        }

    }

}
