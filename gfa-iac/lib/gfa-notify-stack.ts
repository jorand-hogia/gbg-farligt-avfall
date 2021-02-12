import { NestedStack, NestedStackProps } from '@aws-cdk/aws-cloudformation';
import { Construct } from '@aws-cdk/core';
import { IBucket } from '@aws-cdk/aws-s3';
import { functionCreator } from './function-creator';
import { ITable } from '@aws-cdk/aws-dynamodb';
import { ITopic, Topic } from '@aws-cdk/aws-sns';
import { Effect, PolicyStatement } from '@aws-cdk/aws-iam';
import { Rule, Schedule } from '@aws-cdk/aws-events';
import { LambdaFunction } from '@aws-cdk/aws-events-targets';
import { Alarm, ComparisonOperator } from '@aws-cdk/aws-cloudwatch';
import { SnsAction } from '@aws-cdk/aws-cloudwatch-actions';
import { LambdaEndpoint } from './gfa-api-stack';

interface NotifyStackProps extends NestedStackProps {
    version: string,
    artifactsBucket: IBucket,
    eventsTable: ITable,
    alertTopic: ITopic
}

export class NotifyStack extends NestedStack {

    public readonly subscribeEndpoint: LambdaEndpoint;

    constructor(scope: Construct, id: string, props: NotifyStackProps) {
        super(scope, id, props);

        const arrivalToday = new Topic(this, 'gfa-today-topic');

        const newFunction = functionCreator(props.artifactsBucket, props.version);
        const notify = newFunction(this, 'notify', {
            environment: {
                EVENTS_TABLE: props.eventsTable.tableName,
                TODAY_TOPIC: arrivalToday.topicArn,
            }
        });
        props.eventsTable.grantReadData(notify);
        arrivalToday.grantPublish(notify);
        new Rule(this, 'gfa-notify-scheduled-execution', {
            schedule: Schedule.expression('cron(0 3 * * ? *)'),
            targets: [new LambdaFunction(notify)]
        });
        new Alarm(this, 'Notify alert', {
            metric: notify.metricErrors(),
            threshold: 0,
            comparisonOperator: ComparisonOperator.GREATER_THAN_THRESHOLD,
            evaluationPeriods: 1,
            datapointsToAlarm: 1
        }).addAlarmAction(new SnsAction(props.alertTopic));

        const subscribe = newFunction(this, 'subscribe', {
            environment: {
                TODAY_TOPIC: arrivalToday.topicArn,
            }
        });
        subscribe.addToRolePolicy(new PolicyStatement({
            effect: Effect.ALLOW,
            actions: [ 'sns:Subscribe' ],
            resources: [ arrivalToday.topicArn ]
        }));
        this.subscribeEndpoint = {
            lambda: subscribe,
            resource: 'subscriptions',
            methods: ['PUT']
        };
    }
}
