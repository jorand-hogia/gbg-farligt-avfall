# gbg-farligt-avfall
Scrapes times for when the "farligt avfall"-truck and stores them.
End goal is to have subscriptions, and send notifications some time before the truck comes to your subscribed location.

## Requirements
 - Add the following secrets to the github repo:
   - AWS_ACCESS_KEY_ID
   - AWS_SECRET_ACCESS_KEY
   - AWS_REGION
   - S3_ARTIFACT_BUCKET 
 - Add an API key for [MapQuest](https://developer.mapquest.com/) to Secrets Manager (named `mapquest-api-key`) in the AWS account/region you're deploying to

## First deploy
The first time you're deploying this stack you'll need to run the following command:
 - `cdk bootstrap aws://###AWS_ACCOUNT###/###AWS_REGION### -c artifactsBucketName=does_not_matter -c version=does_not_matter`
This is required because the infrastructure containes a nested stack. For CDK to handle this, it needs 'bootstrap' in the AWS account (it will deploy a stagind bucket, where it will place assets, such as nested cloudformation templates). 

## Some useful commands
Launch frontend with 'real' API:
 - `(cd gfa-frontend && API_URL=$(aws cloudformation describe-stacks --stack-name GbgFarligtAvfallStack --query "Stacks[0].Outputs[?OutputKey=='ApiUrl'].OutputValue" --output text) npm run dev)`
