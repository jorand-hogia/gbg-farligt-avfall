import * as lambda from '@aws-cdk/aws-lambda';
import { CfnDBCluster } from '@aws-cdk/aws-rds';
import { Secret } from '@aws-cdk/aws-secretsmanager';
import { App, Duration, Stack, StackProps } from '@aws-cdk/core';

export class GbgFarligtAvfallStack extends Stack {
  public readonly lambdaCode: lambda.CfnParametersCode;
      
  constructor(app: App, id: string, props?: StackProps) {
    super(app, id, props);
    this.lambdaCode = lambda.Code.fromCfnParameters();

    const gfaDbCredentials = new Secret(this, 'gfa-db-credentials', {
      generateSecretString: {
        secretStringTemplate: JSON.stringify({
          username: 'gfa-db'
        }),
        excludePunctuation: true,
        includeSpace: false,
        generateStringKey: 'password'
      }
    });
    const dbName = 'gfa-db';
    const gfaDb = new CfnDBCluster(this, 'gfa-db', {
      engine: 'aurora-postgresql',
      engineMode: 'serverless',
      engineVersion: '10.7',
      enableHttpEndpoint: true, 
      databaseName: dbName,
      masterUsername: gfaDbCredentials.secretValueFromJson('username').toString(),
      masterUserPassword: gfaDbCredentials.secretValueFromJson('password').toString()
    });
    const gfaPoller = new lambda.Function(this, 'gfa-poller', {
      code: this.lambdaCode,
      handler: 'doesnt.matter',
      runtime: lambda.Runtime.PROVIDED,
      timeout: Duration.seconds(10),
      environment: {
        DB_ARN: `arn:${this.partition}:rds:${this.region}:${this.account}:cluster:${gfaDb.ref}`, 
        DB_NAME: dbName,
        DB_CREDENTIALS: gfaDbCredentials.secretArn,
      }
    });

  }
}
