import { Code, CfnParametersCode } from '@aws-cdk/aws-lambda';
import { App, Stack, StackProps } from '@aws-cdk/core';
import { Table, AttributeType, BillingMode } from '@aws-cdk/aws-dynamodb';
import { Bucket } from '@aws-cdk/aws-s3';
import { IngestionStack, IngestionStackProps } from './gfa-ingestion-stack';

export class GbgFarligtAvfallStack extends Stack {
  public readonly scraperCode: CfnParametersCode;
  public readonly saveEventsCode: CfnParametersCode;
  public readonly preProcessStopsCode: CfnParametersCode;
  public readonly saveStopsCode: CfnParametersCode;
      
  constructor(app: App, id: string, props?: StackProps) {
    super(app, id, props);
    this.scraperCode = Code.fromCfnParameters();
    this.saveEventsCode = Code.fromCfnParameters();
    this.preProcessStopsCode = Code.fromCfnParameters();
    this.saveStopsCode = Code.fromCfnParameters();

    const eventsDb = new Table(this, 'gfa-events-db', {
      partitionKey: { name: 'event-date', type: AttributeType.STRING },
      sortKey: { name: 'location-id', type: AttributeType.STRING },
      billingMode: BillingMode.PAY_PER_REQUEST,
    });
    const stopsS3Path = 'stops.json';
    const stopsBucket = new Bucket(this, 'gfa-stops-bucket');

    const ingestionStackProps: IngestionStackProps = {
      scraperCode: this.scraperCode,
      saveEventsCode: this.saveEventsCode,
      preProcessStopsCode: this.preProcessStopsCode,
      saveStopsCode: this.saveStopsCode,
      stopsBucket: stopsBucket,
      stopsPath: stopsS3Path
    }
    const ingestionStack = new IngestionStack(this, 'gfa-ingestion-stack', ingestionStackProps);

  }
}
