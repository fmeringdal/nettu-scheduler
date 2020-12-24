struct RRuleOptions {
    byday: Vec<usize>,
    bymonth: Vec<usize>,
}

struct CalendarEvent {
    startTS: usize,
    endTS: Option<usize>,
    recurrence: Option<RRuleOptions>,
}

impl CalendarEvent {
    pub fn new() -> Self {
        Self {
            startTS: 0,
            endTS: None,
            recurrence: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
