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