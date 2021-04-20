import { Construct, CustomResource, Duration } from "@aws-cdk/core";
import { Function, Runtime, Code } from '@aws-cdk/aws-lambda';
import { Provider } from '@aws-cdk/custom-resources';
import { RetentionDays } from '@aws-cdk/aws-logs';
import { Effect, PolicyStatement } from "@aws-cdk/aws-kms/node_modules/@aws-cdk/aws-iam";
import { HostedZone } from '@aws-cdk/aws-route53';

export interface SendGridDomainVerifierProps {
    hostedZoneId: string,
    domainName: string,
    apiKey: string,
}

export class SendGridDomainVerifier extends Construct {

    constructor(scope: Construct, id: string, props: SendGridDomainVerifierProps) {
        super(scope, id);

        const domainVerifier = new Function(this, `sendgrid-domain-verifier-lambda`, {
            runtime: Runtime.NODEJS_12_X,
            code: Code.fromAsset('lib/sendgrid'),
            handler: 'domain-verifier-lambda.handler',
            timeout: Duration.minutes(6),
        });
        const hostedZone = HostedZone.fromHostedZoneAttributes(this, 'temp-hostedzone', {
            hostedZoneId: props.hostedZoneId,
            zoneName: props.domainName,
        });
        domainVerifier.addToRolePolicy(new PolicyStatement({
            effect: Effect.ALLOW,
            resources: [hostedZone.hostedZoneArn],
            actions: ['route53:ChangeResourceRecordSets']
        }));

        const provider = new Provider(this, 'sendgrid-domain-verifier-provider', {
            onEventHandler: domainVerifier,
            logRetention: RetentionDays.ONE_DAY
        });

        new CustomResource(this, `sendgrid-domain-verifier-${props.domainName}`, {
            serviceToken: provider.serviceToken,
            properties: {
                hostedZoneId: props.hostedZoneId,
                domain: props.domainName,
                apiKey: props.apiKey
            },
        });
    }
}