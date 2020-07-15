import React from 'react';
import logo from './logo.svg';
import './App.css';


class App extends React.Component<any, any> {
  constructor(props: any) {
    super(props);
    this.state = {
      failed: [],
    };
  }
  componentDidMount() {
    fetch('http://localhost:3000/data.json')
      .then(response => response.json())
      .then(result => this.setState({
        failed: result.failed
      }));
  }

  render() {
    console.info(this.state);
    return (
      <div>
        <ul>
          {this.state.failed.map((failure: any) => {
            return (
              <li>{failure.name}</li>
            )
          })}
        </ul>
      </div>
    );
  }
}

export default App;
