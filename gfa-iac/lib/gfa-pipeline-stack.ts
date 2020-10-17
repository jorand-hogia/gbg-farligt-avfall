import * as codebuild from '@aws-cdk/aws-codebuild';
import * as codepipeline from '@aws-cdk/aws-codepipeline';
import * as codepipeline_actions from '@aws-cdk/aws-codepipeline-actions';
import * as lambda from '@aws-cdk/aws-lambda';
import { App, SecretValue, Stack, StackProps } from '@aws-cdk/core';

export interface PipelineStackProps extends StackProps {
  readonly scraperCode: lambda.CfnParametersCode;
  readonly saveEventsCode: lambda.CfnParametersCode;
  readonly preProcessStopsCode: lambda.CfnParametersCode;
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
              'apt update && apt install -y musl-tools'
            ]
          },
          build: {
            commands: [
              'cd gfa-backend',
              '$HOME/.cargo/bin/rustup target add x86_64-unknown-linux-musl',
              `($HOME/.cargo/bin/cargo build --release --target x86_64-unknown-linux-musl)`,
              `cp target/x86_64-unknown-linux-musl/release/scraper scraper/bootstrap`,
              `cp target/x86_64-unknown-linux-musl/release/save-events save-events/bootstrap`,
              `cp target/x86_64-unknown-linux-musl/release/preprocess-stops preprocess-stops/bootstrap`,
            ]    
          },
        },
        artifacts: {
          'secondary-artifacts': {
            'scraper': {
              'base-directory': 'gfa-backend/scraper',
              'files': [
                './bootstrap'
              ]
            },
            'save-events': {
              'base-directory': 'gfa-backend/save-events',
              'files': [
                './bootstrap'
              ]
            },
            'preprocess-stops': {
              'base-directory': 'gfa-backend/preprocess-stops',
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
    const scraperBuildOutput = new codepipeline.Artifact('ScraperBuildOutput');
    const saveEventsBuildOutput = new codepipeline.Artifact('SaveEventsBuildOutput');
    const preProcessStopsBuildOutput = new codepipeline.Artifact('PreProcessStopsBuildOutput');

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
              outputs: [scraperBuildOutput, saveEventsBuildOutput, preProcessStopsBuildOutput],
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
              },
              extraInputs: [scraperBuildOutput, saveEventsBuildOutput, preProcessStopsBuildOutput],
            }),
          ],
        },
      ],
    });
  }
}
