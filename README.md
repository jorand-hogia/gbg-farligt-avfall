# gbg-farligt-avfall
Scrapes times for when the "farligt avfall"-truck and stores them.
End goal is to have subscriptions, and send notifications some time before the truck comes to your subscribed location.

## Requirements
 - Add the following secrets to the github repo:
   - AWS_ACCESS_KEY_ID
   - AWS_SECRET_ACCESS_KEY
   - AWS_REGION
   - S3_ARTIFACT_BUCKET 
   - ADMIN_EMAIL (optional)
   - DOMAIN_NAME (optional)
   - HOSTED_ZONE_ID (optional)

## First deploy
The first time you're deploying this stack you'll need to run the following command:
 - `cdk bootstrap aws://###AWS_ACCOUNT###/###AWS_REGION### -c artifactsBucketName=does_not_matter -c version=does_not_matter`
This is required because the infrastructure containes a nested stack. For CDK to handle this, it needs 'bootstrap' in the AWS account (it will deploy a staging bucket, where it will place assets, such as nested cloudformation templates).

## Some useful commands
Launch frontend with 'real' API:
 - `(cd gfa-frontend && API_URL=$(aws cloudformation describe-stacks --stack-name GbgFarligtAvfallStack --query "Stacks[0].Outputs[?OutputKey=='ApiUrl'].OutputValue" --output text) npm run dev)`

## Adding a new lambda function to the project
 - Create a new folder in `gfa-backend/src`, including a `main.rs` file
 - Update `gfa-backend/Cargo.toml` with a new `[[bin]]` section for the new function
 - Add name of the folder created above to the build step of `.github/workflows/build.yml`:
   - `executables="get-stops save-events save-stops scraper notify subscribe NEW-FOLDER"`
 - Add CDK resource for the new lambda to a suitable stack in `gfa-iac/lib`

 ## Limitations
For sending notifications, AWS SNS is used. Currently a single topic is used, and to only notify relevant subscribers is handled via subscription filters.
Since subscription filters are used, it's quite hard to update an existing subscription. It's possible to do via the AWS Console, but it's hard to design an API for it.
Additionally, AWS SNS have very limited possibilites for changing the e-mail template.

If this were to be used 'for real', I think AWS SNS would need to be replaced with something better suited for sending (transactional?) emails. Preferably with double opt-in, per location that the user subscribes to.
