import { FailedTestSuite, TestCase, } from '../dtos';
import { showDuration } from '../utils';
import React from 'react';

interface FragProps {
    suite: FailedTestSuite
}
interface FragState {
    isExpanded: boolean
}

class FailedTestsFragment extends React.Component<FragProps, FragState>{
    constructor(props: FragProps) {
        super(props)
        this.state = {
            isExpanded: false
        };
    }
    handleClick = () => {
        this.setState((prevState: FragState, _) => { return { isExpanded: !prevState.isExpanded } });
        console.log('this is:', this);
    }

    renderFailedTests = () => {
        if (this.state.isExpanded) {
            return this.props.suite.failedTestcases.map(test => {
                return (
                    <tr key={test.name}>
                        <td colSpan={4}>{test.name}</td>
                        <td>{showDuration(test.time)}</td>
                    </tr>)
            })
        }
    }

    render() {

        return (<>
            <tr key={this.props.suite.name} className={"pure-table-odd"}>
                <td>{this.props.suite.name}</td>
                <td>{this.props.suite.tests}</td>
                <td><a href="#" onClick={this.handleClick}>{this.props.suite.failures}</a></td>
                <td>{this.props.suite.skipped}</td>
                <td>{showDuration(this.props.suite.time)}</td>
            </tr>
            {this.state.isExpanded ? this.props.suite.failedTestcases.map(test => {
                return (
                    <tr key={test.name}>
                        <td colSpan={4}>{test.name}</td>
                        <td>{showDuration(test.time)}</td>
                    </tr>)
            }) : null}

        </>)
    }

}

interface Props {
    failed: FailedTestSuite[]
}

export class Component extends React.Component<Props, any> {
    render() {
        return (
            <section>
                <h2>Failed suites</h2>
                <table className="pure-table">
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Tests</th>
                            <th>Failed</th>
                            <th>Skipped</th>
                            <th>Duration</th>
                        </tr>
                    </thead>
                    {this.props.failed.map(suite => {
                        return (
                            <tbody key={suite.name}>
                                <FailedTestsFragment suite={suite} />
                            </tbody>
                        )
                    })
                    }
                </table >
            </section>
        );
    }
}
