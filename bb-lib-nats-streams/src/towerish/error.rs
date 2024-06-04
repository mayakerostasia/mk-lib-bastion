pub struct FrameSendIssue;

impl std::error::Error for FrameSendIssue {}

impl std::fmt::Display for FrameSendIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Add code here to format the display output
        let _ = f.write_str("FrameSendIssue");
        Ok(())
    }
}

impl std::fmt::Debug for FrameSendIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Add code here to format the debug output
        let _ = f.write_str("FrameSendIssue");
        Ok(())
    }
}
