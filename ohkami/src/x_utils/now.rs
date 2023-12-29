use std::time::{SystemTime, UNIX_EPOCH};


#[inline] pub fn now() -> String {
    let system_now = SystemTime::now().duration_since(UNIX_EPOCH).expect("system time before Unix epoch");
    UTCDateTime::now_from_system(system_now).into_imf_fixdate()
}

#[cfg(test)] mod test {
    fn __correct_now() -> String {
        let mut now = chrono::Utc::now().to_rfc2822(); // like `Wed, 21 Dec 2022 10:16:52 +0000`
        if now.len() == 30 {
            now.insert(5, '0')
        }
        match now.len() {
            31 => now.replace_range(26.., "GMT"),
             _ => unsafe {std::hint::unreachable_unchecked()}
        }
        now
    }

    #[test] fn test_now() {
        let (cn, n) = (__correct_now(), super::now());
        assert_eq!(cn, n);
    }
}


struct UTCDateTime {
    date: Date,
    time: Time,
}
impl UTCDateTime {
    fn now_from_system(system_now: std::time::Duration) -> Self {
        let (secs, nsecs) = (system_now.as_secs() as i64, system_now.subsec_nanos());

        let days = secs.div_euclid(86_400);
        let secs = secs.rem_euclid(86_400);

        let date = Date::from_days(days as i32 + 719_163);
        let time = Time::from_seconds(secs as u32, nsecs);

        Self { date, time }
    }

    fn into_imf_fixdate(self) -> String {
        const IMF_FIXDATE_LEN: usize      = "Sun, 06 Nov 1994 08:49:37 GMT".len();
        const SHORT_WEEKDAYS:  [&str; 7]  = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        const SHORT_MONTHS:    [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

        fn push_hundreds(buf: &mut String, n: u8) {
            debug_assert!(n < 100, "Called `push_hundreds` for `n` greater than 100");
            buf.push((n/10 + b'0') as char);
            buf.push((n%10 + b'0') as char);
        }

        let mut buf = String::with_capacity(IMF_FIXDATE_LEN);
        {
            let Self { date, time } = self;

            buf.push_str(SHORT_WEEKDAYS[date.weekday().num_days_from_sunday() as usize]);
            buf.push_str(", ");

            let day = date.day() as u8;
            if day < 10 {
                buf.push((day + b'0') as char);
            } else {
                push_hundreds(&mut buf, day);
            }

            buf.push(' ');
            buf.push_str(SHORT_MONTHS[date.month0() as usize]);

            buf.push(' ');
            let year = date.year();
            push_hundreds(&mut buf, (year / 100) as u8);
            push_hundreds(&mut buf, (year % 100) as u8);

            buf.push(' ');
            let (hour, min, sec) = time.hms();
            push_hundreds(&mut buf, hour as u8);
            buf.push(':');
            push_hundreds(&mut buf, min as u8);
            buf.push(':');
            let sec = sec + time.nanosecond() / 1_000_000_000;
            push_hundreds(&mut buf, sec as u8);
            
            buf.push_str(" GMT");
        }
        buf
    }
}

struct Time {
    secs: u32,
    frac: u32,
} impl Time {
    const fn from_seconds(secs: u32, nsecs: u32) -> Self {
        debug_assert! {
            secs  < 86_400 &&
            nsecs < 2_000_000_000 &&
            (nsecs < 1_000_000_000 || secs % 60 == 59)
        }
        Self { secs, frac: nsecs }
    }
    const fn hms(&self) -> (u32, u32, u32) {
        let sec = self.secs % 60;
        let mins = self.secs / 60;
        let min = mins % 60;
        let hour = mins / 60;
        (hour, min, sec)
    }
    const fn nanosecond(&self) -> u32 {
        self.frac
    }
}

/// (year << 13) | of
struct Date(i32);
impl Date {
    const fn from_days(days: i32) -> Self {
        const fn cycle_to_yo(cycle: u32) -> (u32, u32) {
            let mut year_mod_400 = cycle / 365;
            let mut ordinal0 = cycle % 365;
            let delta = YEAR_DELTAS[year_mod_400 as usize] as u32;
            if ordinal0 < delta {
                year_mod_400 -= 1;
                ordinal0 += 365 - YEAR_DELTAS[year_mod_400 as usize] as u32;
            } else {
                ordinal0 -= delta;
            }
            (year_mod_400, ordinal0 + 1)
        }

        let days = days + 365;
        let year_div_400 = days.div_euclid(146_097);
        let cycle = days.rem_euclid(146_097);
        let (year_mod_400, ordinal) = cycle_to_yo(cycle as u32);
        let flags = YearFlag::from_year(year_mod_400 as i32);
        Self::from_ordinal_and_flags(year_div_400 * 400 + year_mod_400 as i32, ordinal, flags)
    }
    const fn from_ordinal_and_flags(
        year:    i32,
        ordinal: u32,
        flag:    YearFlag,
    ) -> Date {
        const MAX_YEAR: i32 = i32::MAX >> 13;
        const MIN_YEAR: i32 = i32::MIN >> 13;
        
        debug_assert!(year >= MIN_YEAR && year <= MAX_YEAR, "Year out of range");
        debug_assert!(YearFlag::from_year(year).0 == flag.0);

        let of = Of::new(ordinal, flag);
        Self((year << 13) | (of.0 as i32))
    }

    const fn year(&self) -> i32 {
        self.0 >> 13
    }
    const fn month(&self) -> u32 {
        self.mdf().month()
    }
    const fn month0(&self) -> u32 {
        self.month() - 1
    }
    const fn day(&self) -> u32 {
        self.mdf().day()
    }
    const fn weekday(&self) -> Weekday {
        self.of().weekday()
    }
    const fn mdf(&self) -> Mdf {
        self.of().to_mdf()
    }
    #[inline]
    const fn of(&self) -> Of {
        Of::from_date(self.0)
    }
}

#[derive(Clone, Copy)]
struct YearFlag(u8);
impl YearFlag {
    const fn from_year(year: i32) -> Self {
        const YEAR_TO_FLAG: &[YearFlag; 400] = &[
            BA, G, F, E, DC, B, A, G, FE, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA,
            G, F, E, DC, B, A, G, FE, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G,
            F, E, DC, B, A, G, FE, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F,
            E, DC, B, A, G, FE, D, C, B, AG, F, E, D, // 100
            C, B, A, G, FE, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC,
            B, A, G, FE, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B,
            A, G, FE, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A,
            G, FE, D, C, B, AG, F, E, D, CB, A, G, F, // 200
            E, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A, G, FE,
            D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A, G, FE, D,
            C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A, G, FE, D, C,
            B, AG, F, E, D, CB, A, G, F, ED, C, B, A, // 300
            G, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A, G, FE, D, C, B, AG,
            F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A, G, FE, D, C, B, AG, F,
            E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A, G, FE, D, C, B, AG, F, E,
            D, CB, A, G, F, ED, C, B, A, GF, E, D, C, // 400
        ];
        const A:  YearFlag = YearFlag(0o15);
        const AG: YearFlag = YearFlag(0o05);
        const B:  YearFlag = YearFlag(0o14);
        const BA: YearFlag = YearFlag(0o04);
        const C:  YearFlag = YearFlag(0o13);
        const CB: YearFlag = YearFlag(0o03);
        const D:  YearFlag = YearFlag(0o12);
        const DC: YearFlag = YearFlag(0o02);
        const E:  YearFlag = YearFlag(0o11);
        const ED: YearFlag = YearFlag(0o01);
        const F:  YearFlag = YearFlag(0o17);
        const FE: YearFlag = YearFlag(0o07);
        const G:  YearFlag = YearFlag(0o16);
        const GF: YearFlag = YearFlag(0o06);

        YEAR_TO_FLAG[year.rem_euclid(400) as usize]
    }
}

#[derive(Clone, Copy)]
struct Of(u32);
impl Of {
    const fn new(ordinal: u32, YearFlag(flag): YearFlag) -> Self {
        let of = Self((ordinal << 4) | flag as u32);
        debug_assert!({
            const MIN_OL: u32 = 1 << 1;
            const MAX_OL: u32 = 366 << 1; // `(366 << 1) | 1` would be day 366 in a non-leap year

            let ol = of.0 >> 3;
            MIN_OL <= ol && ol <= MAX_OL
        });
        of
    }
    const fn from_date(date: i32) -> Self {
        Self((date & 0b1_1111_1111_1111) as u32)
    }
    const fn weekday(&self) -> Weekday {
        let Of(of) = *self;
        Weekday::from_u32_mod7((of >> 4) + (of & 0b111))
    }
    const fn to_mdf(&self) -> Mdf {
        Mdf::from_of(*self)
    }
}

struct Mdf(u32);
impl Mdf {
    const fn month(&self) -> u32 {
        let Mdf(mdf) = *self;
        mdf >> 9
    }
    const fn day(&self) -> u32 {
        let Mdf(mdf) = *self;
        (mdf >> 4) & 0b1_1111
    }

    const fn from_of(Of(of): Of) -> Mdf {
        const MAX_OL: u32 = 366 << 1; // `(366 << 1) | 1` would be day 366 in a non-leap year
        const OL_TO_MDL: &[u8; MAX_OL as usize + 1] = &[
            0, 0, // 0
            64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
            64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
            64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, // 1
            66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66,
            66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66,
            66, 66, 66, 66, 66, 66, 66, 66, 66, // 2
            74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72,
            74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72,
            74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, // 3
            76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74,
            76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74,
            76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, // 4
            80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78,
            80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78,
            80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, // 5
            82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80,
            82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80,
            82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, // 6
            86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84,
            86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84,
            86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, // 7
            88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86,
            88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86,
            88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, // 8
            90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88,
            90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88,
            90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, // 9
            94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92,
            94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92,
            94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, // 10
            96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94,
            96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94,
            96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, // 11
            100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100,
            98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98,
            100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100,
            98, // 12
        ];

        let ol = of >> 3;
        if ol <= MAX_OL {
            // Array is indexed from `[1..=MAX_OL]`, with a `0` index having a meaningless value.
            Mdf(of + ((OL_TO_MDL[ol as usize] as u32) << 3))
        } else {
            // Panicking here would be reasonable, but we are just going on with a safe value.
            Mdf(0)
        }
    }
}

#[derive(Clone, Copy)]
enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}
impl Weekday {
    const fn from_u32_mod7(n: u32) -> Self {
        match n % 7 {
            0 => Self::Mon,
            1 => Self::Tue,
            2 => Self::Wed,
            3 => Self::Thu,
            4 => Self::Fri,
            5 => Self::Sat,
            _ => Self::Sun,
        }
    }

    const fn num_days_from_sunday(&self) -> u32 {
        (*self as u32 + 7 - Self::Sun as u32) % 7
    }
}

const YEAR_DELTAS: &[u8; 401] = &[
    0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8,
    8, 9, 9, 9, 9, 10, 10, 10, 10, 11, 11, 11, 11, 12, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 14,
    15, 15, 15, 15, 16, 16, 16, 16, 17, 17, 17, 17, 18, 18, 18, 18, 19, 19, 19, 19, 20, 20, 20, 20,
    21, 21, 21, 21, 22, 22, 22, 22, 23, 23, 23, 23, 24, 24, 24, 24, 25, 25, 25, // 100
    25, 25, 25, 25, 25, 26, 26, 26, 26, 27, 27, 27, 27, 28, 28, 28, 28, 29, 29, 29, 29, 30, 30, 30,
    30, 31, 31, 31, 31, 32, 32, 32, 32, 33, 33, 33, 33, 34, 34, 34, 34, 35, 35, 35, 35, 36, 36, 36,
    36, 37, 37, 37, 37, 38, 38, 38, 38, 39, 39, 39, 39, 40, 40, 40, 40, 41, 41, 41, 41, 42, 42, 42,
    42, 43, 43, 43, 43, 44, 44, 44, 44, 45, 45, 45, 45, 46, 46, 46, 46, 47, 47, 47, 47, 48, 48, 48,
    48, 49, 49, 49, // 200
    49, 49, 49, 49, 49, 50, 50, 50, 50, 51, 51, 51, 51, 52, 52, 52, 52, 53, 53, 53, 53, 54, 54, 54,
    54, 55, 55, 55, 55, 56, 56, 56, 56, 57, 57, 57, 57, 58, 58, 58, 58, 59, 59, 59, 59, 60, 60, 60,
    60, 61, 61, 61, 61, 62, 62, 62, 62, 63, 63, 63, 63, 64, 64, 64, 64, 65, 65, 65, 65, 66, 66, 66,
    66, 67, 67, 67, 67, 68, 68, 68, 68, 69, 69, 69, 69, 70, 70, 70, 70, 71, 71, 71, 71, 72, 72, 72,
    72, 73, 73, 73, // 300
    73, 73, 73, 73, 73, 74, 74, 74, 74, 75, 75, 75, 75, 76, 76, 76, 76, 77, 77, 77, 77, 78, 78, 78,
    78, 79, 79, 79, 79, 80, 80, 80, 80, 81, 81, 81, 81, 82, 82, 82, 82, 83, 83, 83, 83, 84, 84, 84,
    84, 85, 85, 85, 85, 86, 86, 86, 86, 87, 87, 87, 87, 88, 88, 88, 88, 89, 89, 89, 89, 90, 90, 90,
    90, 91, 91, 91, 91, 92, 92, 92, 92, 93, 93, 93, 93, 94, 94, 94, 94, 95, 95, 95, 95, 96, 96, 96,
    96, 97, 97, 97, 97, // 400+1
];
