//! Typed listing projection: scriptable rows; no environment/password output.
use crate::{Error, interface::Active};
use kitty_rc_commands::outputs::{OsWindow, TabInfo, WindowInfo};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    io::{self, Write},
};

#[derive(Debug)]
pub struct Row<'a> {
    pub os: &'a OsWindow,
    pub tab: &'a TabInfo,
    pub window: &'a WindowInfo,
}
pub fn rows(listing: &[OsWindow]) -> impl Iterator<Item = Row<'_>> {
    listing.iter().flat_map(|os| {
        os.tabs.iter().flat_map(move |tab| {
            tab.windows
                .iter()
                .map(move |window| Row { os, tab, window })
        })
    })
}
pub fn value(row: &Row<'_>) -> Value {
    json!({ "id": row.window.id, "title": row.window.title, "pid": row.window.pid,
        "cwd": row.window.cwd, "cmdline": row.window.cmdline,
        "os_window_id": row.os.id, "tab_id": row.tab.id,
        "is_active": row.window.is_active, "is_focused": row.window.is_focused,
        "foreground_processes": row.window.foreground_processes.iter().map(|p| json!({"pid": p.pid, "cmdline": p.cmdline, "cwd": p.cwd})).collect::<Vec<_>>() })
}
pub fn active<'a>(listing: &'a [OsWindow], target: &Active) -> Result<Row<'a>, Error> {
    sampled_active(listing, target)?
        .ok_or_else(|| "no active/focused window returned for selected target".into())
}
/// A valid zero-winner response is normal sampled absence, not an error.
/// Missing facts/targets, filtered global focus and ambiguity remain errors.
pub fn sampled_active<'a>(
    listing: &'a [OsWindow],
    target: &Active,
) -> Result<Option<Row<'a>>, Error> {
    if target.tab.is_none()
        && target.os_window.is_none()
        && (target.scope.match_window.is_some() || target.scope.match_tab.is_some())
    {
        return Err("global focused selection requires an unfiltered listing; use --tab or --os-window for scoped active selection".into());
    }
    let mut ids = BTreeSet::new();
    for row in rows(listing) {
        if !ids.insert(row.window.id) {
            return Err("duplicate window IDs make active selection ambiguous".into());
        }
    }
    let mut candidates = Vec::new();
    if let Some(id) = target.tab {
        let tabs: Vec<_> = listing
            .iter()
            .flat_map(|os| os.tabs.iter().filter(move |tab| tab.id == Some(id)))
            .collect();
        if tabs.len() != 1 {
            return Err(format!("tab {id} is missing or ambiguous in selected response").into());
        }
        for row in rows(listing).filter(|r| r.tab.id == Some(id)) {
            match row.window.is_active {
                Some(true) => candidates.push(row),
                Some(false) => {}
                None => {
                    return Err(
                        "active-window fact missing in selected tab; no winner inferred".into(),
                    );
                }
            }
        }
    } else if let Some(id) = target.os_window {
        let os: Vec<_> = listing.iter().filter(|os| os.id == Some(id)).collect();
        if os.len() != 1 {
            return Err(
                format!("OS window {id} is missing or ambiguous in selected response").into(),
            );
        }
        let mut active_tabs = Vec::new();
        for tab in &os[0].tabs {
            match tab.is_active {
                Some(true) => active_tabs.push(tab),
                Some(false) => {}
                None => {
                    return Err(
                        "active-tab fact missing in selected OS window; no winner inferred".into(),
                    );
                }
            }
        }
        if active_tabs.len() != 1 {
            return Err("selected OS window has absent or ambiguous active tab".into());
        }
        for row in rows(listing).filter(|r| r.os.id == Some(id) && r.tab.id == active_tabs[0].id) {
            match row.window.is_active {
                Some(true) => candidates.push(row),
                Some(false) => {}
                None => return Err("active-window fact missing; no winner inferred".into()),
            }
        }
    } else {
        for row in rows(listing) {
            match row.window.is_focused {
                Some(true) => candidates.push(row),
                Some(false) => {}
                None => return Err(
                    "focused-terminal fact missing; use --tab or --os-window for active selection"
                        .into(),
                ),
            }
        }
    }
    match candidates.len() {
        0 => Ok(None),
        1 => Ok(candidates.pop()),
        n => Err(format!("{n} active/focused windows returned; target is ambiguous").into()),
    }
}
pub fn absence_status(target: &Active) -> &'static str {
    if target.tab.is_some() || target.os_window.is_some() {
        "no-active"
    } else {
        "no-focused"
    }
}
pub fn absent(target: &Active) -> Result<(), Error> {
    let mut out = io::stdout().lock();
    writeln!(out, "-\t({})", absence_status(target))?;
    out.flush()?;
    Ok(())
}
pub fn json_line(value: &Value) -> Result<(), Error> {
    let mut out = io::stdout().lock();
    serde_json::to_writer(&mut out, value)?;
    writeln!(out)?;
    out.flush()?;
    Ok(())
}
pub fn list(listing: &[OsWindow], json_output: bool) -> Result<(), Error> {
    if json_output {
        return json_line(&Value::Array(rows(listing).map(|r| value(&r)).collect()));
    }
    let mut out = io::stdout().lock();
    for row in rows(listing) {
        writeln!(
            out,
            "Window {} (OS {}, tab {}): {}",
            row.window.id.unwrap(),
            row.os.id.unwrap(),
            row.tab.id.unwrap(),
            row.window.title.as_deref().unwrap_or("(untitled)")
        )?;
        writeln!(
            out,
            "  PID: {:?}  CWD: {}  Command: {:?}",
            row.window.pid,
            row.window.cwd.as_deref().unwrap_or("(unknown)"),
            row.window.cmdline
        )?;
        for process in &row.window.foreground_processes {
            writeln!(
                out,
                "  Foreground PID {:?}: {:?} ({})",
                process.pid,
                process.cmdline,
                process.cwd.as_deref().unwrap_or("unknown")
            )?;
        }
    }
    out.flush()?;
    Ok(())
}
pub fn selected(row: &Row<'_>, json_output: bool) -> Result<(), Error> {
    if json_output {
        return json_line(&value(row));
    }
    let mut out = io::stdout().lock();
    writeln!(
        out,
        "{}\t{}",
        row.window.id.unwrap(),
        row.window.title.as_deref().unwrap_or("(untitled)")
    )?;
    out.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn focus_is_not_active_per_tab_and_unknown_never_creates_a_winner() {
        let mut listing: Vec<OsWindow> = serde_json::from_value(json!([{"id":1,"tabs":[{"id":2,"is_active":true,"windows":[{"id":3,"is_active":true,"is_focused":false},{"id":4,"is_active":false,"is_focused":true}]}]}])).unwrap();
        assert_eq!(
            active(&listing, &Active::default()).unwrap().window.id,
            Some(4)
        );
        assert_eq!(
            active(
                &listing,
                &Active {
                    tab: Some(2),
                    ..Default::default()
                }
            )
            .unwrap()
            .window
            .id,
            Some(3)
        );
        listing[0].tabs[0].windows[0].is_focused = None;
        assert!(active(&listing, &Active::default()).is_err());
        listing[0].tabs[0].windows[0].is_focused = Some(true);
        assert!(
            active(&listing, &Active::default())
                .unwrap_err()
                .to_string()
                .contains("ambiguous")
        );
    }

    #[test]
    fn valid_absence_is_nullable_only_for_sampled_selection() {
        assert!(sampled_active(&[], &Active::default()).unwrap().is_none());
        assert!(active(&[], &Active::default()).is_err());
        let listing: Vec<OsWindow> = serde_json::from_value(json!([{"id":1,"tabs":[{"id":2,"windows":[{"id":3,"is_active":false,"is_focused":false}]}]}])).unwrap();
        assert!(
            sampled_active(&listing, &Active::default())
                .unwrap()
                .is_none()
        );
        let target = Active {
            tab: Some(2),
            ..Default::default()
        };
        assert!(sampled_active(&listing, &target).unwrap().is_none());
        assert_eq!(absence_status(&target), "no-active");
        assert!(sampled_active(&[], &target).is_err());
    }
}
