import React from "react";
import { TestSuite } from "../dtos";
import { showDuration } from "../utils";

interface Props {
    all: TestSuite[]
}

export class Component extends React.Component<Props, any> {
    render() {
        return (
            <section>
                <h2>All suites</h2>
                <table className="pure-table pure-table-bordered">
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Tests</th>
                            <th>Failed</th>
                            <th>Skipped</th>
                            <th>Duration</th>
                        </tr>
                    </thead>
                    <tbody>
                        {this.props.all.map(suite => {
                            return (
                                <tr key={suite.name + "-all-tests"}>
                                    <td>{suite.name}</td>
                                    <td>{suite.tests}</td>
                                    <td>{suite.failures}</td>
                                    <td>{suite.skipped}</td>
                                    <td>{showDuration(suite.time)}</td>
                                </tr>
                            )
                        })
                        }
                    </tbody>
                </table >
            </section>
        );
    }
}
