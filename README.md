---
Tags: #readme #directions
---

# mdtask

The goal is to make handling of tasks easier when they are spread across a folder of markdown files. Ultimately I want to move a lot of my custom vim mappings and logic to this binary and integrate it back to neovim (via LSP?).

## Features

* Print all of the unfinished tasks from one or more files or folders (recursively) with default of current folder as root
	* Gives full markdown context for all matches (parent headers, any indented children, tags)

## Usage

`mdtask [folder_or_file(s)]`

## To Do

* [ ] `mdtask` rust tool to manage todos CLI and in nvim
	* could shell to CLI or make it a rust plugin with events or maybe even a LSP with code intel where code fixes would let you mark done/in progress/etc. symbol browser would show tasks in current file or across workspace. this would run in tandem with the zk lsp though. can't think of anything else useful for the lsp so that's probably no good.
	* [ ] `mdtask [folder_or_file(s)]`
		* [x] default to "." for folder @done(2023-02-07 4:46 PM)
		* [x] always recurse if folder is specified @done(2023-02-07 4:46 PM)
		* [x] default grep for `[ ]` @done(2023-02-07 4:46 PM)
		* group by folder and file
			* for each file, put any frontmatter tags in parens next to file name
		* [x] display indented things underneath maybe excluding any `[x]` @done(2023-02-07 4:46 PM)
			* doesn't currently exclude completed items
		* [ ] maybe color things if not piping somewhere
		* [x] default display the context by showing the parent headings (so first preceding heading, then any preceding with fewer `#` and so on) @done(2023-02-07 4:47 PM)
		* [ ] what about parent bullets if indented? probably those too. comment coloring on context stuff
		* example output (should I find a way to add line numbers for easier reference?):
			* `ironcore/writing.md (#blog #writing)`
			* `  # Writing`
			* `  ## Blog Ideas`
			* `  * [ ] How to turn security into a competitive advantage {L:64}`
			* `    * Use examples of trust pages and messaging`
			* `  ## Design Patterns Book`
			* `  * [ ] CMK pattern`
			* `ironcore/IronCore Website.md (#website)`
			* `  # IronCore Website`
			* `  ## Punchlist 2023`
			* `  * [ ] Add KMS Integrations section to SaaS Shield page with Thales logo {L:168}`
	* `mdtask [folder_or_file(s)] --next`
		* show only the first task per file and heading section whether it is undone or in progress
	* `mdtask -i`
		* interactive tui mode? maybe this is less important, but a way to reschedule things and mark as done, etc.
		* hitting enter on a task should launch the editor and `file:linenum`
		* `up`/`down` or `j`/`k` just go between tasks (maybe `gj`/`gk` to go line by line instead)
		* same chords as vim for managing so `td` to mark done, `tt` to schedule to today, etc.
		* `/` to search the list, `f` to change the advanced search params
		* would be super cool to have faceting so each folder in results has a checkbox next to it, each tag too, maybe status, maybe due dates, too, then could pull up advanced filtering and just check the things you want without showing things that aren't applicable (no high prio? don't show that as a filter option)
	* `mdtask [folder(s)_or_file(s)] -t tag1,tag2,tag3`
		* return only items with any of the specified tags in either the frontmatter or the task itself
	* `mdtask [folder(s)_or_file(s)] -t tag1 -t tag2`
		* return only items with all the specified tags in either the frontmatter or the task itself
	* `mdtask [folder(s)_or_file(s)] -p1`
		* filter to high priority tasks with `!!!`
	* `mdtask [folder(s)_or_file(s)] --underway`
		* filter to tasks with `[o]` or `[O]`
	* `mdtask [folder(s)_or_file(s)] --overdue`
		* show overdue tasks: ones with `@due(YYYY-MM-DD)` or ones in a file named either `YYYY-MM-DD.md` or `YYYYMMDD.md` where the date is before today
	* `mdtask [folder(s)_or_file(s)] --due [date]`
		* show tasks due on or before date defaulting to tomorrow
	* `mdtask [folder(s)_or_file(s)] --upcoming`
		* show tasks by due date starting with today and going out from there, group by due date, don't show heading context,
		* put filename and line number in curly braces or not at all?
	* `mdtask [folder_or_file(s)] --done`
		* show completed task by date with most recently completed first. pulls from `@done(YYYY-MM-DD HH:MM)` and groups by day
	* `mdtask [file(s):linenum] --complete`
		* default file is `*`, 
		* default linenum is to complete all incomplete tasks, otherwise just the task at that line number, if any
		* marks as `[x]` and adds `@done(YYYY-MM-DD)` to the end
	* `mdtask [file(s):linenum] --inprogress`
		* marks as `[o]`
	* `mdtask [file(s):linenum] --reschedule [YYYY-MM-DD]`
		* default date is today
		* adds `[>]` and `>date` to source
		* recreates the task in a file named after the target date with `<YYYY-MM-DD` appended if the source file has a date in the frontmatter or filename and if there isn't already such a tag.
		* move also any subbullets maybe excluding completed tasks and those tasks' subbullets
		* _this is tricky because we need some configuration telling us where to find the file to put it in or where to create that file and what format the filename has and a way to override where to find the config file and the config switches too_
	* If I integrated with vim, I think I'd want some kind of query language with some kind of conceal which I could press enter when on top of to execute in a popup window or something. So inside a doc I might have a code section with the code language of `mdtask` and then the query in the code. Maybe hitting enter over that code block would give me a popup. Or maybe I'll be able to do multi-line virtual text and just show the results automatically? Then enter would pop the interactive results or if enter can happen over specific virtual text, then go to task?
		* If I can use virtual text like that, I might also like to display references including inbound links and/or any diaries that had source tasks at top of file? And also any tasks completed on the day of the diary I'm looking at.
	* Pressing `<enter>` on a tag should bring up a Telescope search of that tag -- filter out tasks that have been postponed or canceled though

