//! Lint test: Enforce no direct printing in source code
//!
//! ## Purpose
//!
//! After tree_fmt migration, all output must use TreeFmtFormatter.
//! This test ensures no println!/print!/eprintln! calls exist in src/
//! (except in binary entry points).
//!
//! ## Test Strategy
//!
//! Static analysis of source files to detect violations.
//! Fails loudly with file:line information if violations found.

use std::fs;
use std::path::Path;

#[test]
fn test_no_println_in_source()
{
  let violations = find_printing_violations("src");

  if !violations.is_empty()
  {
    let mut error_msg = String::from(
      "\n❌ VIOLATION: Direct printing detected in source files\n\n"
    );
    error_msg.push_str("All output must use TreeFmtFormatter. Found:\n\n");

    for violation in &violations
    {
      error_msg.push_str(&format!("  {}:{} - {}\n",
        violation.file,
        violation.line,
        violation.call
      ));
    }

    error_msg.push_str("\nFix: Replace with TreeFmtFormatter::format_single() or format_list()\n");

    panic!("{}", error_msg);
  }
}

#[test]
fn test_no_print_in_source()
{
  let src_path = Path::new("src");
  if !src_path.exists()
  {
    // Skip if src/ directory not found (might be running from wrong location)
    return;
  }

  let violations = find_macro_violations("src", "print!");

  if !violations.is_empty()
  {
    let mut error_msg = String::from(
      "\n❌ VIOLATION: Direct print! detected in source files\n\n"
    );

    for violation in &violations
    {
      error_msg.push_str(&format!("  {}:{}\n",
        violation.file,
        violation.line
      ));
    }

    error_msg.push_str("\nFix: Use TreeFmtFormatter instead\n");

    panic!("{}", error_msg);
  }
}

#[test]
fn test_adapters_use_tree_fmt_formatter()
{
  let violations = find_old_formatter_usage("src/adapters");

  if !violations.is_empty()
  {
    let mut error_msg = String::from(
      "\n❌ VIOLATION: Old Formatter detected in adapters\n\n"
    );
    error_msg.push_str("All adapters must use TreeFmtFormatter. Found:\n\n");

    for violation in &violations
    {
      error_msg.push_str(&format!("  {}:{} - {}\n",
        violation.file,
        violation.line,
        violation.call
      ));
    }

    error_msg.push_str("\nFix: Change 'Formatter' to 'TreeFmtFormatter'\n");

    panic!("{}", error_msg);
  }
}

// ============================================================================
// Helper Functions
// ============================================================================

struct Violation
{
  file: String,
  line: usize,
  call: String,
}

fn find_printing_violations(dir: &str) -> Vec<Violation>
{
  let mut violations = Vec::new();
  let path = Path::new(dir);

  if !path.exists()
  {
    return violations;
  }

  visit_dirs(path, &mut |file_path| {
    // Skip binary entry points (they're allowed to use println!)
    if file_path.to_str().unwrap_or("").contains("/bin/")
    {
      return;
    }

    // Skip test files (they can use println! for debugging)
    if file_path.to_str().unwrap_or("").contains("/tests/")
    {
      return;
    }

    if let Ok(content) = fs::read_to_string(file_path)
    {
      for (line_num, line) in content.lines().enumerate()
      {
        // Skip comments
        if line.trim_start().starts_with("//")
        {
          continue;
        }

        // Check for println!, print!, eprintln!, eprint!
        for macro_name in &["println!", "print!", "eprintln!", "eprint!"]
        {
          if line.contains(macro_name)
          {
            violations.push(Violation {
              file: file_path.to_str().unwrap_or("?").to_string(),
              line: line_num + 1,
              call: macro_name.to_string(),
            });
          }
        }
      }
    }
  });

  violations
}

fn find_macro_violations(dir: &str, macro_name: &str) -> Vec<Violation>
{
  let mut violations = Vec::new();
  let path = Path::new(dir);

  if !path.exists()
  {
    return violations;
  }

  visit_dirs(path, &mut |file_path| {
    // Skip binary entry points
    if file_path.to_str().unwrap_or("").contains("/bin/")
    {
      return;
    }

    // Skip test files
    if file_path.to_str().unwrap_or("").contains("/tests/")
    {
      return;
    }

    if let Ok(content) = fs::read_to_string(file_path)
    {
      for (line_num, line) in content.lines().enumerate()
      {
        // Skip comments
        if line.trim_start().starts_with("//")
        {
          continue;
        }

        if line.contains(macro_name)
        {
          violations.push(Violation {
            file: file_path.to_str().unwrap_or("?").to_string(),
            line: line_num + 1,
            call: macro_name.to_string(),
          });
        }
      }
    }
  });

  violations
}

fn find_old_formatter_usage(dir: &str) -> Vec<Violation>
{
  let mut violations = Vec::new();
  let path = Path::new(dir);

  if !path.exists()
  {
    return violations;
  }

  visit_dirs(path, &mut |file_path| {
    if let Ok(content) = fs::read_to_string(file_path)
    {
      for (line_num, line) in content.lines().enumerate()
      {
        // Skip comments
        if line.trim_start().starts_with("//")
        {
          continue;
        }

        // Check for old Formatter import (not TreeFmtFormatter or std::fmt::Formatter)
        if line.contains("use crate::formatting::Formatter;")
          || (line.contains("formatter: &Formatter") && !line.contains("TreeFmtFormatter"))
        {
          violations.push(Violation {
            file: file_path.to_str().unwrap_or("?").to_string(),
            line: line_num + 1,
            call: "Old Formatter".to_string(),
          });
        }
      }
    }
  });

  violations
}

fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&Path))
{
  if dir.is_dir()
  {
    if let Ok(entries) = fs::read_dir(dir)
    {
      for entry in entries.flatten()
      {
        let path = entry.path();
        if path.is_dir()
        {
          visit_dirs(&path, cb);
        }
        else if path.extension().and_then(|s| s.to_str()) == Some("rs")
        {
          cb(&path);
        }
      }
    }
  }
}
