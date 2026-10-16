//! PoliformaT (Sakai LMS) models: Sites, Tasks/Assignments, Announcements.

use super::utils::deserialize_attachments;
use serde::{Deserialize, Serialize};

/// A course or project workspace in PoliformaT.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Site {
    /// Workspace ID.
    pub id: String,

    /// Title of the site or subject.
    pub title: String,

    /// Optional site description.
    #[serde(default)]
    pub description: Option<String>,
}

/// A PoliformaT task/assignment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    /// Unique assignment identifier.
    #[serde(alias = "id", alias = "assignmentId")]
    pub id: String,

    /// Assignment title.
    #[serde(alias = "title")]
    pub title: String,

    /// Associated course or site title.
    #[serde(alias = "site_title", alias = "siteTitle", alias = "context", default)]
    pub site_title: Option<String>,

    /// Due date string (ISO or formatted).
    #[serde(alias = "due_date", alias = "dueTimeString", alias = "dueDate", default)]
    pub due_date: Option<String>,

    /// Detailed instructions for completing the task.
    #[serde(alias = "instructions", default)]
    pub instructions: Option<String>,

    /// Submission status (e.g. "NOT_SUBMITTED", "SUBMITTED", "GRADED").
    #[serde(alias = "status", default)]
    pub status: Option<String>,

    /// List of attachment URLs or filenames.
    #[serde(alias = "attachments", default, deserialize_with = "deserialize_attachments")]
    pub attachments: Vec<String>,
}

/// An announcement posted in a PoliformaT site.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Announcement {
    /// Unique announcement identifier.
    #[serde(alias = "id", alias = "announcementId")]
    pub id: String,

    /// Announcement title or subject.
    #[serde(alias = "title")]
    pub title: String,

    /// Body content (often HTML or plain text).
    #[serde(alias = "body", default)]
    pub body: Option<String>,

    /// Title of the course or site where it was published.
    #[serde(alias = "site_title", alias = "siteTitle", default)]
    pub site_title: Option<String>,

    /// Publication date.
    #[serde(alias = "date", alias = "createdOn", default)]
    pub date: Option<String>,
}
