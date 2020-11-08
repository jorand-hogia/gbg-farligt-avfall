import { Construct, RemovalPolicy } from '@aws-cdk/core';
import { NestedStack, NestedStackProps } from '@aws-cdk/aws-cloudformation';
import { BlockPublicAccess, Bucket } from '@aws-cdk/aws-s3';
import { CfnDistribution, CloudFrontWebDistribution, HttpVersion, OriginAccessIdentity, PriceClass } from '@aws-cdk/aws-cloudfront';

export class WebStack extends NestedStack {

    public readonly webUrl: string;
    public readonly webHostingBucketName: string;

    constructor(scope: Construct, id: string, props?: NestedStackProps) {
        super(scope, id, props);

        const webHostingBucket = new Bucket(this, 'gfa-web-bucket', {
            removalPolicy: RemovalPolicy.DESTROY,
            websiteIndexDocument: 'index.html',
            blockPublicAccess: BlockPublicAccess.BLOCK_ALL,
        });
        this.webHostingBucketName = webHostingBucket.bucketName;
        
        const accessIdentity = new OriginAccessIdentity(this, 'gfa-web-access-identity');
        webHostingBucket.grantRead(accessIdentity);

        const distribution = new CloudFrontWebDistribution(this, 'gfa-web-dist', {
            originConfigs: [
                {
                    s3OriginSource: {
                        s3BucketSource: webHostingBucket,
                        originAccessIdentity: accessIdentity,
                    },
                    behaviors: [{ isDefaultBehavior: true }],
                }
            ],
            priceClass: PriceClass.PRICE_CLASS_100,
            httpVersion: HttpVersion.HTTP2,
            defaultRootObject: 'index.html',
            errorConfigurations: [
                {
                    errorCode: 404,
                    responseCode: 200,
                    responsePagePath: 'index.html',
                }
            ] 
        });
        this.webUrl = distribution.distributionDomainName;
    }
}
