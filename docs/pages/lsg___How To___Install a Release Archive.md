tags:: [[Diataxis/How To]]
see-also:: [[lsg/How To/Install with mise]], [[lsg/How To/Build from Source]], [[lsg/Ref/Release Archives]]

- # How To Install a Release Archive
	- ## Overview
		- Each release on [GitHub Releases](https://github.com/codekiln/logseq-gardener/releases) holds an archive of `lsg` for each platform it builds for, and a SHA-256 checksum for each archive. Download the archive for your computer, check its checksum, and put `lsg` on your PATH. [[lsg/How To/Install with mise]] does these steps for you.
	- ## Prerequisites
		- A computer that releases build for. Each platform has its own archive:
			- | Platform | Archive |
			  | --- | --- |
			  | macOS on Apple silicon | `logseq-gardener-aarch64-apple-darwin.tar.xz` |
			  | macOS on Intel | `logseq-gardener-x86_64-apple-darwin.tar.xz` |
			  | Linux on x86-64 | `logseq-gardener-x86_64-unknown-linux-gnu.tar.xz` |
			  | Windows on x86-64 | `logseq-gardener-x86_64-pc-windows-msvc.zip` |
	- ## Steps
		- ### 1. Download the archive and its checksum
			- Open a release on [GitHub Releases](https://github.com/codekiln/logseq-gardener/releases); the latest is at the top. Download your platform's archive and the file with the same name and `.sha256` added, such as `logseq-gardener-aarch64-apple-darwin.tar.xz.sha256`.
			- With the [GitHub CLI](https://cli.github.com/), this command downloads both files of the latest release for macOS on Apple silicon:
				- ```sh
				  gh release download --repo codekiln/logseq-gardener --pattern 'logseq-gardener-aarch64-apple-darwin.tar.xz*'
				  ```
			- The release also holds `sha256.sum`, which lists the checksums of every archive in one file.
		- ### 2. Check the checksum
			- Compare the archive's SHA-256 hash with the checksum in its `.sha256` file before you extract the archive.
			- On macOS, `shasum -a 256 -c logseq-gardener-aarch64-apple-darwin.tar.xz.sha256` computes the hash and prints the archive's name followed by `OK` when it matches.
			- On Linux, `sha256sum -c logseq-gardener-x86_64-unknown-linux-gnu.tar.xz.sha256` does the same.
			- In PowerShell, `Get-FileHash -Algorithm SHA256 logseq-gardener-x86_64-pc-windows-msvc.zip` prints the hash. Compare it with the first word of the `.sha256` file, ignoring letter case.
			- If the hash differs from the checksum, delete the archive and download it again.
		- ### 3. Extract the archive
			- On macOS or Linux, run `tar -xf` with the archive's name, such as `tar -xf logseq-gardener-aarch64-apple-darwin.tar.xz`. In PowerShell, run `Expand-Archive logseq-gardener-x86_64-pc-windows-msvc.zip`.
			- The folder it creates has the archive's name. It holds `lsg` (`lsg.exe` on Windows), the license text, the third-party notices, the README, the changelog and the documentation in `docs`.
		- ### 4. Put lsg on your PATH
			- Move `lsg` into a folder on your PATH, such as `~/.local/bin`, and run `lsg version`.
			- If you give copies of `lsg` to others, keep the license and notice files with it. [[lsg/Ref/Release Archives]] describes what each archive holds and how releases are built.
