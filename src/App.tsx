import { useState } from 'react'
import React, { Component } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { Menu } from 'primereact/menu'
import { MenuItem } from 'primereact/menuitem'
import reactLogo from './assets/react.svg'
import './App.css'

class Collection {
  name: string;
  animations: Array<Animation>;

  constructor(name: string, animations: Array<Animation>) {
    this.name = name;
    this.animations = animations;
  }
}

class Animation {
  currentFrameIndex: number;
  currentTime: number;
  duration: number;
  fps: number;
  frames: Array<string>;
  loopStart: number;
  name: string;
  numFrames: number;

  constructor(frames: Array<string>, fps: number, loopStart: number, name: string) {
    this.currentFrameIndex = 0;
    this.currentTime = 0;
    this.duration = frames.length * (1.0 / fps);
    this.frames = frames;
    this.fps = fps;
    this.loopStart = loopStart;
    this.name = name;
    this.numFrames = frames.length;
  }
}

export default class App extends Component {
  state = {
    collectionNames: [] as Array<string>,
    currentAnimation: null as Animation | null,
    currentCollection: null as Collection | null,
  }

  constructor(props: any) {
    super(props)
  }

  componentDidMount(): void {
    invoke('get_collection_list').then(collectionList => {
      this.state.collectionNames = collectionList as Array<string>
      this.state.collectionNames.forEach(name => console.log("Collection name: " + name))
      invoke('get_collection', { collectionName: this.state.collectionNames[0] })
        .then(collection => {
          this.state.currentCollection = collection as Collection
          console.log("Got collection: " + this.state.currentCollection.name)
          this.state.currentAnimation = this.state.currentCollection.animations[0]
          console.log("Current anim: " + this.state.currentAnimation.name)
        });
    })
  }

  render(): React.ReactNode {
    return (
      <div className="App">
        <div className="grid">
          <Menu model={this.getCollections()} />
          <Menu model={this.getAnimations()} />
          <Menu model={this.getFrames()} />
        </div>
        <div>
          <a href="https://vitejs.dev" target="_blank">
            <img src="/vite.svg" className="logo" alt="Vite logo" />
          </a>
          <a href="https://reactjs.org" target="_blank">
            <img src={reactLogo} className="logo react" alt="React logo" />
          </a>
        </div>
        <h1>Vite + React</h1>
        <div className="card">
          <p>
            Edit <code>src/App.tsx</code> and save to test HMR
          </p>
        </div>
        <p className="read-the-docs">
          Click on the Vite and React logos to learn more
        </p>
      </div>
    )
  }

  getCollections() {
    let items: Array<MenuItem> = []
    this.state.collectionNames.forEach(name => items.push({ label: name }))
    return items
  }

  getAnimations() {
    if (this.state.currentCollection == null) {
      return []
    }

    console.log("Current collection is not null: " + JSON.stringify(this.state.currentCollection))

    let items: Array<MenuItem> = [];
    this.state.currentCollection.animations.forEach(animation => items.push({ label: animation.name }))
    return items
  }

  getFrames() {
    if (this.state.currentAnimation == null) {
      return []
    }

    console.log("Current animation is not null: " + JSON.stringify(this.state.currentAnimation))

    let items: Array<MenuItem> = [];
    this.state.currentAnimation.frames.forEach(frame => items.push({ label: frame }))
    return items
  }
}