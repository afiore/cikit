import { FailedTestCase as FailedTestCaseDto, FailedTestSuite } from '../dtos';
import { Component as FailedTestCaseComponent } from './FailedTestCase';
import { showDuration } from '../utils';
import React from 'react';

interface FragProps {
    failedTestcases: FailedTestCaseDto[];
    isExpanded: boolean;
}

const FailedTestsFragment = (props: FragProps) => {
    return (<>
        {props.isExpanded ? props.failedTestcases.map(test => {
            return (
                <FailedTestCaseComponent testCase={test} />)
        }) : null}

    </>)
}


interface Props {
    failed: FailedTestSuite[]
}
interface State {
    expandedSuite?: string
}

export class Component extends React.Component<Props, State> {
    constructor(props: Props) {
        super(props);
        this.state = {
            expandedSuite: undefined
        }
    }
    handleOnClick = (suiteName: string) => {
        this.setState((prevState, _) => {
            return prevState.expandedSuite === suiteName ?
                {
                    expandedSuite: undefined
                } : { expandedSuite: suiteName }
        });
    }
    render() {
        return (
            <section>
                <h2>Failed suites</h2>
                <table className={"pure-table pure-table-bordered"}>
                    <thead>
                        <tr>
                            <th key="name">Name</th>
                            <th key="tests">Tests</th>
                            <th key="failed">Failed</th>
                            <th key="skipped">Skipped</th>
                            <th key="duration">Duration</th>
                        </tr>
                    </thead>
                    <tbody>
                        {this.props.failed.map(suite => {
                            return (
                                <>
                                    <tr key={suite.name}>
                                        <td>{suite.name}</td>
                                        <td>{suite.tests}</td>
                                        <td className="failedcount"><button title="toggle test cases" onClick={() => this.handleOnClick(suite.name)}>{suite.failures}</button></td>
                                        <td>{suite.skipped}</td>
                                        <td>{showDuration(suite.time)}</td>
                                    </tr>

                                    <FailedTestsFragment failedTestcases={suite.failedTestcases} isExpanded={this.state.expandedSuite === suite.name} />
                                </>
                            )
                        })
                        }
                    </tbody>
                </table >
            </section>
        );
    }
}
