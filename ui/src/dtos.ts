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
    failedTestcases: TestCase[];
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
