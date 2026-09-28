mod analysis_tests;
mod dataset_tests;
mod realdb_tests;
mod school_report_tests;
mod student_tests;
mod template_tests;
mod thresholds_tests;

use rusqlite::Connection;

pub fn test_conn() -> Connection {
    crate::db::create_in_memory().expect("in-memory db")
}
