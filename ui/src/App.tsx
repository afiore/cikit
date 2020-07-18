import React from 'react';
import './App.css';
import humanizeDuration from 'humanize-duration';

interface FailedTestcase {
  name: string;
  time: number;
}

interface FailedTestcases {
  testcases: FailedTestcase[];
}

interface FailedTestSuite {
  name: string;
  time: number;
  tests: number;
  failures: number;
  timestamp: Date;
  failedTestcases: FailedTestcase[];
}

interface FailedSuites {
  failed: FailedTestSuite[];
}

const FailedTestsFragment = ({ testcases }: FailedTestcases) => (
  <React.Fragment>
    {
      testcases.map(test => {
        return (<tr key={test.name}>
          <td colSpan={3}>{test.name}</td>
          <td>{humanizeDuration(test.time, { units: ["s", "ms"] })}</td>
        </tr>)
      })
    }
  </React.Fragment>
)

class App extends React.Component<any, FailedSuites> {
  constructor(props: any) {
    super(props);

    this.state = {
      failed: [],
    };
  }
  componentDidMount() {
    fetch('/data.json')
      .then(response => response.json())
      .then(result => this.setState({
        failed: result.failed
      }));
  }

  render() {
    console.info(this.state);
    return (
      <table className="pure-table">
        <thead>
          <tr>
            <th>Name</th>
            <th>Tests</th>
            <th>Failed</th>
            <th>Duration</th>
          </tr>
        </thead>
        {this.state.failed.map(suite => {
          return (
            <tbody>
              <tr key={suite.name} className={"pure-table-odd"}>
                <td>{suite.name}</td>
                <td>{suite.tests}</td>
                <td>{suite.failures}</td>
                <td>{humanizeDuration(suite.time, { units: ["s", "ms"] })}</td>
              </tr>
              <FailedTestsFragment testcases={suite.failedTestcases} />
            </tbody>
          )
        })
        }
      </table >
    );
  }
}

export default App;
