This tool syncs Zenhub Workspace issues to an organization level Github Project.

Requirements:

- Requires a `ZENHUB_TOKEN` and `GITHUB_TOKEN` in the environment. I'm using a `.tokens` file with `env $(cat .tokens) cargo r --release` for now. The github token needs Issues:RW, Pull Requests:RW, Issue Types:RW, and Org:Projects:RW permissions.
- You must have created your desired GitHub organization's Project already and any fields you'd like to sync over. This tool won't create projects or fields.

This currently requires a few hardcoded settings, for now you'll have to read the code to see them, but it is usable. Eventually all those settings should move out into a config file that the application expects or is passed. This tool does not check for the difference between the Zenhub Workspace and the GitHub project before starting work, so if run back to back it will repeatedly set things to values they already are on the GitHub Project.


TODO:

- [ ] support paging, if the lane is over ~100 issues right now, anything past that doesn't get sync'd
- [ ] BUG: sub-issues (closed and open, 9 of 200) were added to "Ungroomed" without estimates. All were from one repo in this case. Some did have estimates in ZH.
- [ ] take a config file with mappings and other information, expand configurability
- [ ] get the issues in the github project already and diff them with the pipeline, so we only make mutation calls for items we need to make changes to
- [ ] add process for updating the `schema` files
- [ ] add support for blocking and connected issues (epics) via adding them as sub-issues in github
- [ ] get iterations syncing over. This is low priority since as current information that's not a very big lift to do manually
