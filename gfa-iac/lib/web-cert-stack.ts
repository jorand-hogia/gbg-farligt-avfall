import { Construct, Stack, StackProps } from "@aws-cdk/core";
import { Certificate } from '@aws-cdk/aws-certificatemanager';
import { CertificateValidation } from '@aws-cdk/aws-certificatemanager';
import { HostedZone } from '@aws-cdk/aws-route53';
import { StringParameter } from "@aws-cdk/aws-ssm";

export interface WebCertStackProps extends StackProps {
    certParameterName: string
}

export class WebCertStack extends Stack {
    constructor(scope: Construct, id: string, props: WebCertStackProps) {
        super(scope, id, props);

        const domainName = scope.node.tryGetContext('domainName');
        const hostedZoneId = scope.node.tryGetContext('hostedZoneId');
        const webDomainName = `gfa.${domainName}`;

        const hostedZone = HostedZone.fromHostedZoneAttributes(this, 'e-hostedzone', {
            hostedZoneId: hostedZoneId,
            zoneName: domainName,
        });

        const webCert = new Certificate(this, 'web-certificate', {
            domainName: webDomainName,
            validation: CertificateValidation.fromDns(hostedZone),
        });

        new StringParameter(this, 'web-certificate-parameter', {
            parameterName: props.certParameterName,
            stringValue: webCert.certificateArn
        });
    }
}
