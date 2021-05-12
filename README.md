# gbg-farligt-avfall
The purpose of this repository is to send e-mail notifications when the [Göteborg Farligt Avfall truck](https://goteborg.se/wps/portal/start/avfall-och-atervinning/har-lamnar-hushall-avfall/farligtavfallbilen/farligt-avfall-bilen) is about to arrive to a subscribed location.

The service is deployed to AWS, and exposes the following API endpoints:
 - GET /stops
    - Returns all stops (streets) which the Göteborg Farligt Avfall traffic
 - PUT /subscriptions
    - Add a new subscription
 - POST /subscriptions/verify?email={email}&auth_token={token}
    - Confirm a previously added subscription
 - DELETE /subscriptions?email={email}&unsubscribe_token={token}
    - Delete a previously added subscription

Included in the service is also a frontend which displays a list of stops, and give the user possibility to subscribe to e-mail notifications for any of the stops.

## Requirements
If you'd like to deploy your own instance of this service, fork the repo and add the following github secrets:
 - AWS_ACCESS_KEY_ID
 - AWS_SECRET_ACCESS_KEY
 - AWS_REGION
 - S3_ARTIFACT_BUCKET 
 - DOMAIN_NAME
 - HOSTED_ZONE_ID
 - SENDGRID_API_KEY
 - ADMIN_EMAIL (optional)

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
