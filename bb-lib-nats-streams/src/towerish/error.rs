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

pub struct MonkeyStartFailure(pub String); 


impl std::error::Error for MonkeyStartFailure {}
impl std::fmt::Display for MonkeyStartFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = f.write_str(format!("MonkeyStartFailure -> {}", &self.0).as_str());
        Ok(())
    }
}

impl std::fmt::Debug for MonkeyStartFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = f.write_str(format!("MonkeyStartFailure -> {}", &self.0).as_str());
        Ok(())
    }
}

