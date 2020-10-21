import * as codebuild from '@aws-cdk/aws-codebuild';
import * as codepipeline from '@aws-cdk/aws-codepipeline';
import * as codepipeline_actions from '@aws-cdk/aws-codepipeline-actions';
import * as lambda from '@aws-cdk/aws-lambda';
import { App, SecretValue, Stack, StackProps } from '@aws-cdk/core';

export interface PipelineStackProps extends StackProps {
  readonly scraperCode: lambda.CfnParametersCode;
  readonly saveEventsCode: lambda.CfnParametersCode;
  readonly preProcessStopsCode: lambda.CfnParametersCode;
  readonly saveStopsCode: lambda.CfnParametersCode;
  readonly repoName: string
  readonly repoOwner: string
  readonly githubToken: string
}

export class GbgFarligtAvfallPipelineStack extends Stack {
  constructor(app: App, id: string, props: PipelineStackProps) {
    super(app, id, props);

    const lambdaBuild = new codebuild.Project(this, 'LambdaBuild', {
      buildSpec: codebuild.BuildSpec.fromObject({
        version: '0.2',
        phases: {
          install: {
            commands: [
              'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y',
              'apt update && apt install -y musl-tools',
              '$HOME/.cargo/bin/rustup target add x86_64-unknown-linux-musl',
            ]
          },
          build: {
            commands: [
              'cd gfa-backend',
              '$HOME/.cargo/bin/cargo test --release',
              `$HOME/.cargo/bin/cargo build --release --target x86_64-unknown-linux-musl`,
            ]
          },
          post_build: {
            commands: [
              'cp target/x86_64-unknown-linux-musl/release/scraper src/scraper/bootstrap',
              'cp target/x86_64-unknown-linux-musl/release/save-events src/save-events/bootstrap',
              'cp target/x86_64-unknown-linux-musl/release/preprocess-stops src/preprocess-stops/bootstrap',
              'cp target/x86_64-unknown-linux-musl/release/save-stops src/save-stops/bootstrap',
            ]
          }
        },
        artifacts: {
          'secondary-artifacts': {
            'ScraperOutput': {
              'base-directory': 'gfa-backend/src/scraper',
              'files': [
                './bootstrap'
              ]
            },
            'SaveEventsOutput': {
              'base-directory': 'gfa-backend/src/save-events',
              'files': [
                './bootstrap'
              ]
            },
            'PreProcessStopsOutput': {
              'base-directory': 'gfa-backend/src/preprocess-stops',
              'files': [
                './bootstrap'
              ]
            },
            'SaveStopsBuildOutput': {
              'base-directory': 'gfa-backend/src/save-stops',
              'files': [
                './bootstrap'
              ]
            }
          }
        },
      }),
      environment: {
        buildImage: codebuild.LinuxBuildImage.STANDARD_2_0,
      }
    });

    const cdkBuild = new codebuild.PipelineProject(this, 'CdkBuild', {
      buildSpec: codebuild.BuildSpec.fromObject({
        version: '0.2',
        phases: {
          install: {
            commands: [
              'cd gfa-iac',
              'npm install'
            ]
          },
          build: {
            commands: [
              'npm run build',
              'npm run cdk synth -- -o dist'
            ],
          },
        },
        artifacts: {
          'base-directory': 'gfa-iac/dist',
          files: [
            'GbgFarligtAvfallStack.template.json',
          ],
        },
      }),
      environment: {
        buildImage: codebuild.LinuxBuildImage.STANDARD_2_0,
      },
    });

    const sourceOutput = new codepipeline.Artifact();
    const cdkBuildOutput = new codepipeline.Artifact('CdkBuildOutput');
    const scraperBuildOutput = new codepipeline.Artifact('ScraperOutput');
    const saveEventsBuildOutput = new codepipeline.Artifact('SaveEventsOutput');
    const preProcessStopsBuildOutput = new codepipeline.Artifact('PreProcessStopsOutput');
    const saveStopsBuildOutput = new codepipeline.Artifact('SaveStopsBuildOutput');

    new codepipeline.Pipeline(this, 'Pipeline', {
      stages: [
        {
          stageName: 'Source',
          actions: [
            new codepipeline_actions.GitHubSourceAction({
              actionName: 'GitHub_Source',
              owner: props.repoOwner, 
              repo: props.repoName, 
              branch: 'master', // TODO: Parameterize?
              oauthToken: SecretValue.plainText(props.githubToken),
              output: sourceOutput,
            }),
          ],
        },
        {
          stageName: 'Build',
          actions: [
            new codepipeline_actions.CodeBuildAction({
              actionName: 'LambdaBuild',
              project: lambdaBuild,
              input: sourceOutput,
              outputs: [scraperBuildOutput, saveEventsBuildOutput, preProcessStopsBuildOutput, saveStopsBuildOutput],
            }),
            new codepipeline_actions.CodeBuildAction({
              actionName: 'CDK_Build',
              project: cdkBuild,
              input: sourceOutput,
              outputs: [cdkBuildOutput],
            }),
          ],
        },
        {
          stageName: 'Deploy',
          actions: [
            new codepipeline_actions.CloudFormationCreateUpdateStackAction({
              actionName: 'Gfa_Deploy',
              templatePath: cdkBuildOutput.atPath('GbgFarligtAvfallStack.template.json'),
              stackName: 'GbgFarligtAvfallStack',
              adminPermissions: true,
              parameterOverrides: {
                ...props.scraperCode.assign(scraperBuildOutput.s3Location),
                ...props.saveEventsCode.assign(saveEventsBuildOutput.s3Location),
                ...props.preProcessStopsCode.assign(preProcessStopsBuildOutput.s3Location),
                ...props.saveStopsCode.assign(saveStopsBuildOutput.s3Location),
              },
              extraInputs: [scraperBuildOutput, saveEventsBuildOutput, preProcessStopsBuildOutput, saveStopsBuildOutput],
            }),
          ],
        },
      ],
    });
  }
}
