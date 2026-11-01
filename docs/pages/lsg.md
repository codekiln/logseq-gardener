alias:: [[Logseq Gardener]]
tags:: [[Diataxis/Explanation]]

- # Logseq Gardener
	- Logseq Gardener (`lsg`) is a command-line tool for people and coding agents who keep notes in a Logseq Markdown garden. It publishes chosen namespaces of a garden as a static website that reads like a site Logseq 0.10 publishes, and it prints the source file of a page named by its title, alias or journal date. It reads the garden and leaves every file in it unchanged.
	- ## Get started
		- {{embed ((23ecf0c0-bef8-45bf-8fa8-26a273de5901))}}
		- [[lsg/How To/Install with mise]] shows how to pin a version and what mise checks when it installs `lsg`. [[lsg/How To/Install a Release Archive]] and [[lsg/How To/Build from Source]] install `lsg` without mise.
		- [[lsg/Tutorial/Publish Part of a Public Garden]] walks you through publishing part of a public garden and reading the site in a browser.
	- ## Publish a garden
		- [[lsg/How To/Publish Selected Namespaces]]
		- [[lsg/How To/Customize a Published Site]]
		- [[lsg/How To/Publish a Garden to GitHub Pages]]
		- [[lsg/How To/Add a Site Plugin]]
		- [[lsg/Static Publishing]] explains what `lsg publish` does with a garden.
	- ## Look something up
		- [[lsg/Ref/Command Line]] lists the commands, their JSON output and their exit codes.
		- [[lsg/Ref/Publish Options]] lists the options of `lsg publish`.
		- [[lsg/Ref/Published Pages]] and [[lsg/Ref/Site Navigation]] describe what a published site shows.
		- [[lsg/Ref/Logseq Compatibility]] says what `lsg` publishes as Logseq does and where it differs.
	- ## Contribute
		- [[lsg/How To/Contribute a Change]] shows how to take a change from plan to pull request.
		- [[lsg/How To/Edit the Documentation]] shows how to edit these pages and build the site.
		- [[lsg/Architecture]] explains how the SDK and the command line divide the work.
		- [[lsg/Ref/Project Checks]] lists the checks every pull request passes.
	- ## Kinds of pages
		- Each page serves one reader need, and its title and tag name the need: a how-to page for a task, a tutorial for learning by doing, a reference page for looking something up, and an explanation for understanding why. [[Diataxis]] lists the kinds, and the Hierarchy section below lists every page.
