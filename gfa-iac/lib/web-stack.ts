import { Construct, RemovalPolicy, Duration } from '@aws-cdk/core';
import { NestedStack, NestedStackProps } from '@aws-cdk/aws-cloudformation';
import { BlockPublicAccess, Bucket } from '@aws-cdk/aws-s3';
import { CloudFrontAllowedCachedMethods, CloudFrontAllowedMethods, CloudFrontWebDistribution, Distribution, HttpVersion, OriginAccessIdentity, PriceClass } from '@aws-cdk/aws-cloudfront';
import { HostedZone, ARecord, RecordTarget } from '@aws-cdk/aws-route53';
import * as targets from '@aws-cdk/aws-route53-targets';
import { AwsCustomResource, AwsCustomResourcePolicy, PhysicalResourceId } from '@aws-cdk/custom-resources';

export interface WebStackProps extends NestedStackProps {
  webCertParameterName: string
}

export class WebStack extends NestedStack {

    public readonly webDistributionId: string;
    public readonly webHostingBucketName: string;
    public readonly externalDomain: string;

    private readonly rootDomainName: string;

    constructor(scope: Construct, id: string, props: WebStackProps) {
        super(scope, id, props);
        this.rootDomainName = scope.node.tryGetContext('domainName');
        this.externalDomain = `gfa.${this.rootDomainName}`;

        const webHostingBucket = this.setupHostingBucket();
        this.webHostingBucketName = webHostingBucket.bucketName;
        
        const webCertArn = this.getCertificateArn(props.webCertParameterName);

        const distribution = this.setupCloudFrontDist(webHostingBucket, webCertArn);
        this.webDistributionId = distribution.distributionId;

        this.setupDnsRecord(scope.node.tryGetContext('hostedZoneId'), this.rootDomainName, distribution);
    }

    setupHostingBucket() {
        return new Bucket(this, 'web-bucket', {
            removalPolicy: RemovalPolicy.DESTROY,
            websiteIndexDocument: 'index.html',
            blockPublicAccess: BlockPublicAccess.BLOCK_ALL,
        });
    }

    getCertificateArn(parameterName: string) {
        const certificateArn = new AwsCustomResource(this, 'get-web-certificate-arn', {
            onUpdate: {
                service: 'SSM',
                action: 'getParameter',
                parameters: {
                    'Name': parameterName,
                },
                region: 'us-east-1',
                physicalResourceId: PhysicalResourceId.of(new Date().toISOString()),
            },
            policy: AwsCustomResourcePolicy.fromSdkCalls({
                resources: AwsCustomResourcePolicy.ANY_RESOURCE,
            }),
        });
        return certificateArn.getResponseField('Parameter.Value');
    }

    setupCloudFrontDist(hostingBucket: Bucket, certificateArn: string) {
        const accessIdentity = new OriginAccessIdentity(this, 'web-access-identity');
        hostingBucket.grantRead(accessIdentity);
        return new CloudFrontWebDistribution(this, 'web-dist', {
            originConfigs: [
                {
                    s3OriginSource: {
                        s3BucketSource: hostingBucket,
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
                aliases: [ this.externalDomain ],
                props: {
                    acmCertificateArn: certificateArn,
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
    }

    setupDnsRecord(hostedZoneId: string, domainName: string, cloudFront: CloudFrontWebDistribution) {
        const hostedZone = HostedZone.fromHostedZoneAttributes(this, 'e-hostedzone', {
            hostedZoneId: hostedZoneId,
            zoneName: domainName,
        });
        new ARecord(this, 'web-domain-record', {
            zone: hostedZone,
            target: RecordTarget.fromAlias(new targets.CloudFrontTarget(cloudFront)),
            recordName: this.externalDomain,
        });
    }
}
