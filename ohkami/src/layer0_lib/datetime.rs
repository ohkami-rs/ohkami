#[inline] pub fn now() -> String {
    let mut now = chrono::Utc::now().to_rfc2822(); // like `Wed, 21 Dec 2022 10:16:52 +0000`
    match now.len() {
        30 => now.replace_range(25.., "GMT"),
        31 => now.replace_range(26.., "GMT"),
         _ => unsafe {std::hint::unreachable_unchecked()}
    }
    now
}

/*
chrono::Utc::now

    pub fn now() -> DateTime<Utc> {
        let now   = ::std::time::SystemTime::now().duration_since(UNIX_EPOCH).expect("system time before Unix epoch");
        let naive = NaiveDateTime::from_timestamp_opt(now.as_secs() as i64, now.subsec_nanos()).unwrap();
        Utc.from_utc_datetime(&naive)
    }

    pub const UNIX_EPOCH: SystemTime = SystemTime(time::UNIX_EPOCH);
*/

/*
chrono::NativeDateTime

    pub struct NaiveDateTime {
        date: NaiveDate,
        time: NaiveTime,
    }

    pub fn from_timestamp_opt(secs: i64, nsecs: u32) -> Option<NaiveDateTime> {
        let days = secs.div_euclid(86_400);
        let secs = secs.rem_euclid(86_400);
        let date = i32::try_from(days)
            .ok()
            .and_then(|days| days.checked_add(719_163))
            .and_then(NaiveDate::from_num_days_from_ce_opt);
        let time = NaiveTime::from_num_seconds_from_midnight_opt(secs as u32, nsecs);
        match (date, time) {
            (Some(date), Some(time)) => Some(NaiveDateTime { date, time }),
            (_, _) => None,
        }
    }

*/

/*
chrono::DateTime

    pub struct DateTime<Tz: TimeZone> {
        datetime: NaiveDateTime,
        offset: Tz::Offset,
    }

    fn from_utc_datetime(&self, utc: &NaiveDateTime) -> DateTime<Self> {
        DateTime::from_naive_utc_and_offset(*utc, self.offset_from_utc_datetime(utc))
    }
    pub fn from_naive_utc_and_offset(datetime: NaiveDateTime, offset: Tz::Offset) -> DateTime<Tz> {
        DateTime { datetime, offset }
    }
*/

/* chrono::NativeDate

    pub struct NaiveDate {
        ymdf: DateImpl, // (year << 13) | of
    }

    pub const fn from_num_days_from_ce_opt(days: i32) -> Option<NaiveDate> {
        let days = try_opt!(days.checked_add(365)); // make December 31, 1 BCE equal to day 0
        let year_div_400 = days.div_euclid(146_097);
        let cycle = days.rem_euclid(146_097);
        let (year_mod_400, ordinal) = internals::cycle_to_yo(cycle as u32);
        let flags = YearFlags::from_year_mod_400(year_mod_400 as i32);
        NaiveDate::from_ordinal_and_flags(year_div_400 * 400 + year_mod_400 as i32, ordinal, flags)
    }
*/

/*
chrono::NativeTime

    pub struct NaiveTime {
        secs: u32,
        frac: u32,
    }

    pub const fn from_num_seconds_from_midnight_opt(secs: u32, nano: u32) -> Option<NaiveTime> {
        if secs >= 86_400 || nano >= 2_000_000_000 || (nano >= 1_000_000_000 && secs % 60 != 59) {
            return None;
        }
        Some(NaiveTime { secs, frac: nano })
    }

*/

/* ========== */

/* chrono::DateTime::to_rfc_2822

    pub fn to_rfc2822(&self) -> String {
        let mut result = String::with_capacity(32);
        crate::format::write_rfc2822(&mut result, self.naive_local(), self.offset.fix())
            .expect("writing rfc2822 datetime to string should never fail");
        result
    }

    pub(crate) fn write_rfc2822(
        w: &mut impl Write,
        dt: NaiveDateTime,
        off: FixedOffset,
    ) -> fmt::Result {
        write_rfc2822_inner(w, dt.date(), dt.time(), off, default_locale())
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    /// write datetimes like `Tue, 1 Jul 2003 10:52:37 +0200`, same as `%a, %d %b %Y %H:%M:%S %z`
    fn write_rfc2822_inner(
        w: &mut impl Write,
        d: NaiveDate,
        t: NaiveTime,
        off: FixedOffset,
        locale: Locale,
    ) -> fmt::Result {
        let year = d.year();
        // RFC2822 is only defined on years 0 through 9999
        if !(0..=9999).contains(&year) {
            return Err(fmt::Error);
        }

        w.write_str(short_weekdays(locale)[d.weekday().num_days_from_sunday() as usize])?;
        w.write_str(", ")?;
        let day = d.day();
        if day < 10 {
            w.write_char((b'0' + day as u8) as char)?;
        } else {
            write_hundreds(w, day as u8)?;
        }
        w.write_char(' ')?;
        w.write_str(short_months(locale)[d.month0() as usize])?;
        w.write_char(' ')?;
        write_hundreds(w, (year / 100) as u8)?;
        write_hundreds(w, (year % 100) as u8)?;
        w.write_char(' ')?;

        let (hour, min, sec) = t.hms();
        write_hundreds(w, hour as u8)?;
        w.write_char(':')?;
        write_hundreds(w, min as u8)?;
        w.write_char(':')?;
        let sec = sec + t.nanosecond() / 1_000_000_000;
        write_hundreds(w, sec as u8)?;
        w.write_char(' ')?;
        OffsetFormat {
            precision: OffsetPrecision::Minutes,
            colons: Colons::None,
            allow_zulu: false,
            padding: Pad::Zero,
        }
        .format(w, off)
    }
    
    /// Equivalent to `{:02}` formatting for n < 100.
    pub(crate) fn write_hundreds(w: &mut impl Write, n: u8) -> fmt::Result {
        if n >= 100 {
            return Err(fmt::Error);
        }
    
        let tens = b'0' + n / 10;
        let ones = b'0' + n % 10;
        w.write_char(tens as char)?;
        w.write_char(ones as char)
    }
*/
