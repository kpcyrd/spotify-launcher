use crate::errors::*;
use std::collections::HashMap;
use std::env;

#[derive(Debug, PartialEq)]
pub enum Architecture {
    Amd64,
    I386,
    Unknown(&'static str),
}

impl Architecture {
    pub const fn current() -> Architecture {
        if cfg!(target_arch = "x86_64") {
            Architecture::Amd64
        } else if cfg!(target_arch = "x86") {
            Architecture::I386
        } else {
            Architecture::Unknown(env::consts::ARCH)
        }
    }

    pub const fn to_debian_str(&self) -> &str {
        match self {
            Architecture::Amd64 => "amd64",
            Architecture::I386 => "i386",
            Architecture::Unknown(arch) => arch,
        }
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct Release {
    pub architectures: Vec<String>,
    pub sha256_sums: HashMap<String, String>,
}

pub fn parse_release_file(data: &str) -> Result<Release> {
    let mut section = None;
    let mut release = Release::default();

    for line in data.lines() {
        trace!("Release file parser got line: {:?}", line);

        if let Some(line) = line.strip_prefix(' ') {
            if section == Some("SHA256:") {
                let (hash, line) = line
                    .split_once(' ')
                    .context("Malformed sha256 line in release file")?;
                let (_, file) = line
                    .rsplit_once(' ')
                    .context("Malformed sha256 line in release file")?;

                debug!(
                    "Adding file hash from release file, {:?} => {:?}",
                    file, hash
                );
                release
                    .sha256_sums
                    .insert(file.to_string(), hash.to_string());
            }
        } else if line.ends_with(':') {
            section = Some(line);
        } else if let Some((key, value)) = line.split_once(": ") {
            if key == "Architectures" {
                let list = value.split(' ').map(String::from);
                release.architectures = list.collect();
            }
        }
    }

    Ok(release)
}

#[derive(Debug, PartialEq)]
pub struct Pkg {
    pub package: String,
    pub version: String,
    pub filename: String,
    pub sha256sum: String,
}

impl Pkg {
    pub fn download_url(&self) -> String {
        format!("http://repository.spotify.com/{}", self.filename)
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct NewPkg {
    package: Option<String>,
    version: Option<String>,
    filename: Option<String>,
    sha256sum: Option<String>,
}

impl NewPkg {
    pub fn set(&mut self, key: &str, value: String) {
        match key {
            "Package" => self.package = Some(value),
            "Version" => self.version = Some(value),
            "Filename" => self.filename = Some(value),
            "SHA256" => self.sha256sum = Some(value),
            _ => (),
        }
    }
}

pub fn parse_package_index(data: &str) -> Result<Vec<Pkg>> {
    let mut out = Vec::new();
    let mut pkg: Option<NewPkg> = None;

    for line in data.lines().chain([""]) {
        trace!("Package index parser got line: {:?}", line);

        if line.is_empty() {
            if let Some(pkg) = pkg.take() {
                out.push(Pkg {
                    package: pkg.package.context("Missing field: `package`")?,
                    version: pkg.version.context("Missing field: `version`")?,
                    filename: pkg.filename.context("Missing field: `filename`")?,
                    sha256sum: pkg.sha256sum.context("Missing field: `sha256sum`")?,
                });
            }
        } else {
            if line.starts_with(' ') || line.ends_with(':') {
                // not supported
                continue;
            }

            let (key, value) = line
                .split_once(": ")
                .with_context(|| anyhow!("Line does not have key-value format: {:?}", line))?;

            pkg.get_or_insert_with(NewPkg::default)
                .set(key, value.to_string());
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_release() -> Result<()> {
        let data = r#"Origin: Spotify LTD
Label: Spotify Public Repository
Suite: testing
Codename: testing
Version: 0.4
Date: Wed, 27 Apr 2022 12:30:15 UTC
Architectures: amd64 i386
Components: non-free
Description: Spotify's repository for beta releases of the desktop client
MD5Sum:
 edf4635027ed7a5df78633d70657834e 1220 non-free/binary-amd64/Packages
 77247f419f4652944f6f7e674356eab5 665 non-free/binary-amd64/Packages.gz
 896ad45b10babce26185deb30607ef41 197 non-free/binary-amd64/Release
 fddbbdbabd9e75aa9435609890419f0b 1067 non-free/binary-i386/Packages
 8684c7481e8bbaa79771e64374fd615b 628 non-free/binary-i386/Packages.gz
 fce0f84291a9cdabd4458a2e1af891ba 196 non-free/binary-i386/Release
 d41d8cd98f00b204e9800998ecf8427e 0 non-free/source/Sources
 7029066c27ac6f5ef18d660d5741979a 20 non-free/source/Sources.gz
 ea17201e3c955b95e948af6c95a5f698 198 non-free/source/Release
SHA1:
 71f7a00f6f8f16677396a5366df3b1f288d7e8d5 1220 non-free/binary-amd64/Packages
 cf491797f25ec68e852ae85c3f3836175523a3c1 665 non-free/binary-amd64/Packages.gz
 4faadfa7ad9ad0ec07f8f17f31963878f5fe02df 197 non-free/binary-amd64/Release
 da9551bbd5defe5de3a2673d04782e69609518cc 1067 non-free/binary-i386/Packages
 76a712620b1483e31111492d4abd1cf8b76b73b9 628 non-free/binary-i386/Packages.gz
 b63d39ad880704ae8bf33b57504e25728068e49b 196 non-free/binary-i386/Release
 da39a3ee5e6b4b0d3255bfef95601890afd80709 0 non-free/source/Sources
 46c6643f07aa7f6bfe7118de926b86defc5087c4 20 non-free/source/Sources.gz
 3c5cc8f592faad9f57a4045cd6193615786ae98d 198 non-free/source/Release
SHA256:
 7eb86d0a8bbbfb356b2c641f039214ad30f7f5d7faabdf546d5f83d4f0f574cd 1220 non-free/binary-amd64/Packages
 8c55f74c379873d3b4bb63b7e05eff20705e9f08348b93b9e20d3baa8d27d383 665 non-free/binary-amd64/Packages.gz
 35876d5aa96d00b39fe3a26660a7665b4ced399a650dc3344b20dee7ad3cc766 197 non-free/binary-amd64/Release
 497184ddb1dc525de81a1bd98ac97175b176d15ead076680d3e8d27b0b5329c8 1067 non-free/binary-i386/Packages
 2654df365b0dd96a6307e6e56780815d8695ed89da547c1a6beb509035a8ddd0 628 non-free/binary-i386/Packages.gz
 1b0d97da546cdcd460e99eaadaac929048b217a9e043a2a919a648910e8f1be4 196 non-free/binary-i386/Release
 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 0 non-free/source/Sources
 59869db34853933b239f1e2219cf7d431da006aa919635478511fabbfc8849d2 20 non-free/source/Sources.gz
 963be5cb6b84350b820fd4bb4ce5059bb4cbb16dcfa7614c630ded76a09155a5 198 non-free/source/Release
"#;
        let parsed = parse_release_file(data)?;
        assert_eq!(parsed, {
            let mut release = Release {
                architectures: vec!["amd64".to_string(), "i386".to_string()],
                ..Release::default()
            };
            let m = &mut release.sha256_sums;
            m.insert(
                "non-free/binary-amd64/Packages".into(),
                "7eb86d0a8bbbfb356b2c641f039214ad30f7f5d7faabdf546d5f83d4f0f574cd".into(),
            );
            m.insert(
                "non-free/binary-amd64/Packages.gz".into(),
                "8c55f74c379873d3b4bb63b7e05eff20705e9f08348b93b9e20d3baa8d27d383".into(),
            );
            m.insert(
                "non-free/binary-amd64/Release".into(),
                "35876d5aa96d00b39fe3a26660a7665b4ced399a650dc3344b20dee7ad3cc766".into(),
            );
            m.insert(
                "non-free/binary-i386/Packages".into(),
                "497184ddb1dc525de81a1bd98ac97175b176d15ead076680d3e8d27b0b5329c8".into(),
            );
            m.insert(
                "non-free/binary-i386/Packages.gz".into(),
                "2654df365b0dd96a6307e6e56780815d8695ed89da547c1a6beb509035a8ddd0".into(),
            );
            m.insert(
                "non-free/binary-i386/Release".into(),
                "1b0d97da546cdcd460e99eaadaac929048b217a9e043a2a919a648910e8f1be4".into(),
            );
            m.insert(
                "non-free/source/Sources".into(),
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into(),
            );
            m.insert(
                "non-free/source/Sources.gz".into(),
                "59869db34853933b239f1e2219cf7d431da006aa919635478511fabbfc8849d2".into(),
            );
            m.insert(
                "non-free/source/Release".into(),
                "963be5cb6b84350b820fd4bb4ce5059bb4cbb16dcfa7614c630ded76a09155a5".into(),
            );
            release
        });
        Ok(())
    }

    #[test]
    fn test_parse_package_index() -> Result<()> {
        let data = r#"Package: spotify-client
Architecture: amd64
Version: 1:1.1.84.716.gc5f8b819
Priority: extra
Section: sound
Maintainer: Spotify <tux@spotify.com>
Installed-Size: 291694
Depends: libasound2, libatk-bridge2.0-0, libatomic1, libcurl3-gnutls, libgbm1, libgconf-2-4, libglib2.0-0, libgtk-3-0, libnss3, libssl3 | libssl1.1 | libssl1.0.2 | libssl1.0.1 | libssl1.0.0, libxshmfence1, libxss1, libxtst6, xdg-utils
Recommends: libavcodec58 | libavcodec-extra58 | libavcodec57 | libavcodec-extra57 | libavcodec-ffmpeg56 | libavcodec-ffmpeg-extra56 | libavcodec54 | libavcodec-extra-54, libavformat58 | libavformat57 | libavformat-ffmpeg56 | libavformat54
Suggests: libnotify4
Filename: pool/non-free/s/spotify-client/spotify-client_1.1.84.716.gc5f8b819_amd64.deb
SHA512: 3cc25f28ae791ac26607117a5df668f803ed8e58f0ace085010a6242fdde97766bdc1c752560850795c9b4324f3e019937fe9af2788a1946ebb70ee781f50d99
Homepage: https://www.spotify.com
Size: 119770140
SHA256: 08e6b2666dc2a39624890e553a3046d05ecebe17bcc2fe930d49314b2fb812c7
SHA1: 987258467c50076490400b2539688f0808b86ebb
MD5sum: 57c7e2f950b25ea26328abf4b232555a
Description: Spotify streaming music client
License: https://www.spotify.com/legal/end-user-agreement
Vendor: Spotify AB
"#;
        let parsed = parse_package_index(data)?;
        assert_eq!(
            parsed,
            &[Pkg {
                package: "spotify-client".into(),
                version: "1:1.1.84.716.gc5f8b819".into(),
                filename:
                    "pool/non-free/s/spotify-client/spotify-client_1.1.84.716.gc5f8b819_amd64.deb"
                        .into(),
                sha256sum: "08e6b2666dc2a39624890e553a3046d05ecebe17bcc2fe930d49314b2fb812c7"
                    .into(),
            },]
        );
        Ok(())
    }
}
