use crate::issue::Issue;
use anyhow::Result;
use runtime::Plugin;
use std::path::Path;

pub trait IssuePlugin: Plugin {
    /// Called after an issue is successfully created.
    fn on_issue_created(&self, _project_dir: &Path, _issue: &Issue) -> Result<()> {
        Ok(())
    }

    /// Called after an issue is moved to a new status.
    fn on_issue_moved(
        &self,
        _project_dir: &Path,
        _issue: &Issue,
        _from: &str,
        _to: &str,
    ) -> Result<()> {
        Ok(())
    }

    /// Called after an issue is deleted.
    fn on_issue_deleted(&self, _project_dir: &Path, _issue_name: &str) -> Result<()> {
        Ok(())
    }
}

pub struct IssuePluginRegistry {
    plugins: Vec<Box<dyn IssuePlugin>>,
}

impl IssuePluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn IssuePlugin>) {
        self.plugins.push(plugin);
    }

    pub fn on_issue_created(&self, project_dir: &Path, issue: &Issue) {
        for plugin in &self.plugins {
            if let Err(e) = plugin.on_issue_created(project_dir, issue) {
                eprintln!("[ship:{}] on_issue_created: {}", plugin.name(), e);
            }
        }
    }

    pub fn on_issue_moved(&self, project_dir: &Path, issue: &Issue, from: &str, to: &str) {
        for plugin in &self.plugins {
            if let Err(e) = plugin.on_issue_moved(project_dir, issue, from, to) {
                eprintln!("[ship:{}] on_issue_moved: {}", plugin.name(), e);
            }
        }
    }

    pub fn on_issue_deleted(&self, project_dir: &Path, issue_name: &str) {
        for plugin in &self.plugins {
            if let Err(e) = plugin.on_issue_deleted(project_dir, issue_name) {
                eprintln!("[ship:{}] on_issue_deleted: {}", plugin.name(), e);
            }
        }
    }
}
