import { Construct, RemovalPolicy } from '@aws-cdk/core';
import { NestedStack, NestedStackProps } from '@aws-cdk/aws-cloudformation';
import { Bucket } from '@aws-cdk/aws-s3';
import { CloudFrontWebDistribution, HttpVersion, PriceClass } from '@aws-cdk/aws-cloudfront';

export class WebStack extends NestedStack {

    public readonly webHostingBucket: Bucket;
    public readonly webUrl: string;

    constructor(scope: Construct, id: string, props?: NestedStackProps) {
        super(scope, id, props);

        this.webHostingBucket = new Bucket(this, 'gfa-web-bucket', {
            removalPolicy: RemovalPolicy.DESTROY,
            websiteIndexDocument: 'index.html'
        });

        const distribution = new CloudFrontWebDistribution(this, 'gfa-web-dist', {
            originConfigs: [
                {
                    s3OriginSource: {
                        s3BucketSource: this.webHostingBucket,
                    },
                    behaviors: [{ isDefaultBehavior: true }],
                }
            ],
            priceClass: PriceClass.PRICE_CLASS_100,
            httpVersion: HttpVersion.HTTP2,
            defaultRootObject: 'index.html',
        });
        this.webUrl = distribution.distributionDomainName;
    }
}
