# gbg-farligt-avfall
Some description that I will write later.

## Requirements
Add GITHUB_TOKEN to `gfa-iac/.env`. The value should be a personal access token with the following permissions:

- **repo** - to read the repository
- **admin:repo_hook** - to create webhook

This token is used by AWS CodePipeline to trigger on new commits.

## Inspiration
https://docs.aws.amazon.com/cdk/latest/guide/codepipeline_example.html

## Deploy pipeline stack
- `cd gfa iac`
- `cdk deploy GbgFarligtAvfallPipelineStack`

## Deploy main stack
Not needed! The deploy pipeline will trigger on commits to master.

## TODO
 - ?