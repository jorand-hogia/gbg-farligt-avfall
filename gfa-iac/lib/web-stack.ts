import { Construct, RemovalPolicy, Duration } from '@aws-cdk/core';
import { NestedStack, NestedStackProps } from '@aws-cdk/aws-cloudformation';
import { BlockPublicAccess, Bucket } from '@aws-cdk/aws-s3';
import { CloudFrontAllowedCachedMethods, CloudFrontAllowedMethods, CloudFrontWebDistribution, HttpVersion, OriginAccessIdentity, PriceClass } from '@aws-cdk/aws-cloudfront';
import { HostedZone, ARecord, RecordTarget } from '@aws-cdk/aws-route53';
import * as targets from '@aws-cdk/aws-route53-targets';
import { AwsCustomResource, AwsCustomResourcePolicy, PhysicalResourceId } from '@aws-cdk/custom-resources';

export class WebStack extends NestedStack {

    public readonly webUrl: string;
    public readonly webDistributionId: string;
    public readonly webHostingBucketName: string;

    constructor(scope: Construct, id: string, props?: NestedStackProps) {
        super(scope, id, props);

        const webHostingBucket = new Bucket(this, 'web-bucket', {
            removalPolicy: RemovalPolicy.DESTROY,
            websiteIndexDocument: 'index.html',
            blockPublicAccess: BlockPublicAccess.BLOCK_ALL,
        });
        this.webHostingBucketName = webHostingBucket.bucketName;
        
        const accessIdentity = new OriginAccessIdentity(this, 'web-access-identity');
        webHostingBucket.grantRead(accessIdentity);


        const certificateArn = new AwsCustomResource(this, 'get-web-certificate-arn', {
            onUpdate: {
                service: 'SSM',
                action: 'getParameter',
                parameters: {
                    'Name': 'gfa-web-certificate',
                },
                region: 'us-east-1',
                physicalResourceId: PhysicalResourceId.of(new Date().toISOString()),
            },
            policy: AwsCustomResourcePolicy.fromSdkCalls({
                resources: AwsCustomResourcePolicy.ANY_RESOURCE,
            }),
        });
        const domainName = scope.node.tryGetContext('domainName');
        const webDomainName = `gfa.${domainName}`;

        const distribution = new CloudFrontWebDistribution(this, 'web-dist', {
            originConfigs: [
                {
                    s3OriginSource: {
                        s3BucketSource: webHostingBucket,
                        originAccessIdentity: accessIdentity,
                    },
                    behaviors: [{
                        isDefaultBehavior: true,
                        cachedMethods: CloudFrontAllowedCachedMethods.GET_HEAD_OPTIONS,
                        allowedMethods: CloudFrontAllowedMethods.GET_HEAD_OPTIONS,
                        compress: true,
                        defaultTtl: Duration.hours(1),
                        maxTtl: Duration.days(1),
                        minTtl: Duration.minutes(1),
                    }],
                }
            ],
            viewerCertificate: {
                aliases: [ webDomainName ],
                props: {
                    acmCertificateArn: certificateArn.getResponseField('Parameter.Value'),
                    sslSupportMethod: 'sni-only',
                },
            },
            priceClass: PriceClass.PRICE_CLASS_100,
            httpVersion: HttpVersion.HTTP2,
            defaultRootObject: 'index.html',
            errorConfigurations: [
                {
                    errorCode: 403,
                    responseCode: 200,
                    responsePagePath: '/index.html',
                    errorCachingMinTtl: 86400
                },
                {
                    errorCode: 404,
                    responseCode: 200,
                    responsePagePath: '/index.html',
                    errorCachingMinTtl: 86400
                }
            ] 
        });
        this.webUrl = distribution.distributionDomainName;
        this.webDistributionId = distribution.distributionId;

        const hostedZoneId = scope.node.tryGetContext('hostedZoneId');
        const hostedZone = HostedZone.fromHostedZoneAttributes(this, 'e-hostedzone', {
            hostedZoneId: hostedZoneId,
            zoneName: domainName,
        });
        new ARecord(this, 'web-domain-record', {
            zone: hostedZone,
            target: RecordTarget.fromAlias(new targets.CloudFrontTarget(distribution)),
            recordName: webDomainName,
        });
    }
}
