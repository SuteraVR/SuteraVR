#[cfg(test)]
mod test {
    use crate::clocking::{
        oneshot_headers::OneshotHeader, sutera_header::SuteraHeader, sutera_status::SuteraStatus,
        traits::ClockingFrame,
    };

    #[test]
    fn schema_snapshot() {
        insta::assert_debug_snapshot!(SuteraHeader::MIN_FRAME_SIZE);
        insta::assert_debug_snapshot!(SuteraHeader::MAX_FRAME_SIZE);

        insta::assert_debug_snapshot!(SuteraStatus::MIN_FRAME_SIZE);
        insta::assert_debug_snapshot!(SuteraStatus::MAX_FRAME_SIZE);

        insta::assert_debug_snapshot!(OneshotHeader::MIN_FRAME_SIZE);
        insta::assert_debug_snapshot!(OneshotHeader::MAX_FRAME_SIZE);
    }
}
