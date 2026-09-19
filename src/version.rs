//! Conversion between Java version strings and proto's semver format.

use proto_pdk_api::VersionSpec;

// #############################################################################
// MARK: Proto to Java version

/// Converts the semver representation used by proto back to Foojay's Java
/// version representation. Foojay uses a fourth numeric component for a few
/// vendors, which proto encodes in the pre-release field.
pub fn to_java_version(spec: &VersionSpec) -> String {
  match spec {
    VersionSpec::Canary => "canary".into(),
    VersionSpec::Alias(alias) => alias.to_string(),
    _ => {
      let version = spec.as_version().expect("version spec must be a version");
      let mut output = version.major.to_string();

      // A leading numeric prerelease component stores Java's fourth number.
      let (vendor, pre) = match version.prerelease.as_deref() {
        Some(value) => {
          let (first, rest) = value.split_once('.').unwrap_or((value, ""));

          if !first.is_empty() && first.bytes().all(|byte| byte.is_ascii_digit()) {
            (Some(first), (!rest.is_empty()).then_some(rest))
          } else {
            (None, Some(value))
          }
        }
        None => (None, None),
      };

      // Preserve zero components when they precede a fourth number.
      if version.minor > 0 || version.patch > 0 || vendor.is_some() {
        output.push('.');
        output.push_str(&version.minor.to_string());

        if version.patch > 0 || vendor.is_some() {
          output.push('.');
          output.push_str(&version.patch.to_string());
        }
      }

      if let Some(vendor) = vendor {
        output.push('.');
        output.push_str(vendor);
      }

      if let Some(pre) = pre {
        output.push('-');
        output.push_str(pre);
      }

      if let Some(build) = &version.build {
        output.push('+');
        output.push_str(build);
      }

      output
    }
  }
}

// #############################################################################
// MARK: Java to proto version

/// Converts Foojay's Java-version values to a proto-compatible semver value.
pub fn from_java_version(value: &str) -> String {
  let mut value = value;
  let mut build = None;
  let mut pre = None;

  // Strip build metadata first because prerelease parsing uses the remaining text.
  if let Some(index) = value.rfind('+') {
    build = Some(&value[index + 1..]);
    value = &value[..index];
  }

  if let Some(index) = value.find('-') {
    pre = Some(&value[index + 1..]);
    value = &value[..index];
  }

  let mut parts = value.split('.');
  let major = parts
    .next()
    .expect("Foojay versions require a major component");
  let minor = parts.next().unwrap_or("0");
  let patch = parts.next().unwrap_or("0");
  let vendor = parts.next();

  // Semver permits only three core numbers: 21.0.11.2-ea becomes 21.0.11-2.ea.
  let mut output = format!("{major}.{minor}.{patch}");
  if let Some(vendor) = vendor {
    output.push('-');
    output.push_str(vendor);
  }

  if let Some(pre) = pre {
    output.push(if vendor.is_some() { '.' } else { '-' });
    output.push_str(pre);
  }

  if let Some(build) = build {
    output.push('+');
    output.push_str(build);
  }

  output
}

// #############################################################################
// MARK: Tests

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn normalizes_foojay_versions_for_proto() {
    assert_eq!(from_java_version("25"), "25.0.0");
    assert_eq!(from_java_version("25.0.3"), "25.0.3");
    assert_eq!(from_java_version("21.0.11+10"), "21.0.11+10");
    assert_eq!(from_java_version("25-ea+1"), "25.0.0-ea+1");
  }

  #[test]
  fn restores_versions_used_in_foojay_queries() {
    assert_eq!(
      to_java_version(&VersionSpec::parse("25.0.3").unwrap()),
      "25.0.3"
    );
    assert_eq!(
      to_java_version(&VersionSpec::parse("21.0.11+10").unwrap()),
      "21.0.11+10"
    );
  }

  #[test]
  fn preserves_fourth_component_and_release_metadata() {
    for (java, proto) in [
      ("21.0.0.1", "21.0.0-1"),
      ("21.0.11.2+10", "21.0.11-2+10"),
      ("21.0.11.2-ea+10", "21.0.11-2.ea+10"),
    ] {
      assert_eq!(from_java_version(java), proto);
      assert_eq!(to_java_version(&VersionSpec::parse(proto).unwrap()), java);
    }
  }
}
