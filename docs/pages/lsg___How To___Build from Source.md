tags:: [[Diataxis/How To]]
see-also:: [[lsg/How To/Install with mise]], [[lsg/How To/Contribute a Change]]

- # How To Build from Source
	- ## Overview
		- Build `lsg` from a checkout of the repository when you want a change that no release holds yet. mise installs the pinned Rust toolchain and build tools, and Cargo builds `lsg`.
	- ## Prerequisites
		- [mise](https://mise.jdx.dev/) and git.
		- The native linker for your platform: Xcode command-line tools on macOS, a C build toolchain on Linux, or Visual Studio C++ build tools on Windows.
	- ## Steps
		- ### 1. Get the source and its tools
			- ```sh
			  git clone https://github.com/codekiln/logseq-gardener.git
			  cd logseq-gardener
			  mise install
			  ```
			- mise may ask you to trust the checked-out configuration before it installs the pinned tools. Run `mise trust`, then `mise install` again.
		- ### 2. Build lsg
			- Run `mise run build:release`. It creates `target/release/lsg`, or `target/release/lsg.exe` on Windows. The first build downloads development tools and dependencies.
			- Run `./target/release/lsg help` to check the build. On Windows, run `target/release/lsg.exe help`.
		- ### 3. Put lsg on your PATH
			- Run `mise exec -- cargo install --path . --locked`. It installs `lsg` into Cargo's binary folder, usually `~/.cargo/bin`.
			- Add that folder to PATH and run `lsg version`.
	- ## Troubleshooting
		- If the build fails when it links `lsg`, install the native linker for your platform from the prerequisites.
		- The contributor tasks, such as `mise run ci`, use Bash, and CI runs them on Linux. [[lsg/Ref/Project Checks]] lists them.
