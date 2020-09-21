# gbg-farligt-avfall
Some description that I will write later.

## Requirements
The AWS account where you deploy this must have a secret called `github-token`.
It should contain a personal access token with the following permissions:

- **repo** - to read the repository
- **admin:repo_hook** - to create webhook

This token is used by AWS CodePipeline to trigger on new commits.

