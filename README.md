The `.graphql` schema files will likely need to be updated from time to time. TODO add process here.

Requires a `ZENHUB_TOKEN` and `GITHUB_TOKEN` in the environment. I'm using a `.tokens` file with `env $(cat .tokens) cargo r --release` for now. The github token needs Issues:RW, Pull Requests:RW, Issue Types:RW, and Projects:RW permissions.

Old migrator call: `env $(cat ~/code/oss/zenhub-to-github-migrator/.tokens) projectsmigrator https://github.com/orgs/IronCoreLabs/projects/8 --workspace="The Big Board" -x="Pipeline:Next Sprint" -x="Pipeline:This Sprint" -f="Estimate:Estimate" -f="Priority:Priority" -f="Pipeline:Status" -f="Linked Issues:Text" -x="Epic:*" -f="Blocking:Text" -f="Sprint:Iteration"`
