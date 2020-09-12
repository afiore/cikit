export interface Failure {
    type: String;
    stackTrace?: String;
    message?: String;
}

export interface FailedTestCase extends TestCase {
    failure: Failure;
}

export interface TestCase {
    name: string;
    time: number;
}

export interface TestSuite {
    name: string;
    time: number;
    tests: number;
    failures: number;
    skipped: number;
    timestamp: Date;
}

export interface FailedTestSuite extends TestSuite {
    failedTestcases: FailedTestCase[];
}

export interface Summary {
    time: number;
    tests: number;
    failures: number;
    errors: number;
    skipped: number;
}

interface PullRequest {
    title: string;
    htmlUrl: string;
}

interface GithubUser {
    avatarUrl: string;
    login: string;
    htmlUrl: string;
}

export interface GithubContext {
    number: number;
    pullRequest: PullRequest,
    sender: GithubUser
}
