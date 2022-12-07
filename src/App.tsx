import { Component } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import reactLogo from './assets/react.svg'
import './App.css'
import SelectableList from './components/SelectableList'
import 'primereact/resources/themes/mdc-dark-indigo/theme.css'

class Collection {
  name: string
  animations: Animation[]

  constructor(name: string, animations: Animation[]) {
    this.name = name
    this.animations = animations
  }
}

class Animation {
  currentFrameIndex: number
  currentTime: number
  duration: number
  fps: number
  frames: string[]
  loopStart: number
  name: string
  numFrames: number

  constructor(frames: string[], fps: number, loopStart: number, name: string) {
    this.currentFrameIndex = 0
    this.currentTime = 0
    this.duration = frames.length * (1.0 / fps)
    this.frames = frames
    this.fps = fps
    this.loopStart = loopStart
    this.name = name
    this.numFrames = frames.length
  }
}

interface AppProps {

}

interface AppState {
  collectionNames: string[]
  currentAnimation: Animation | null
  currentCollection: Collection | null
  currentFrame: string | null
}

export default class App extends Component<AppProps, AppState> {
  constructor(props: AppProps) {
    super(props)
    this.state = {
      collectionNames: [],
      currentAnimation: null,
      currentCollection: null,
      currentFrame: null,
    }
  }

  componentDidMount() {
    invoke('get_collection_list').then(collectionList => {
      this.setState({ collectionNames: collectionList as string[] }, () => {
        if (this.state.collectionNames.length > 0) {
          invoke('get_collection', { collectionName: this.state.collectionNames[0] })
            .then(collection => {
              if (collection != null) {
                this.setState({ currentCollection: collection as Collection }, () => {
                  if (this.state.currentCollection != null &&
                    this.state.currentCollection.animations.length > 0) {
                    this.setState({ currentAnimation: this.state.currentCollection.animations[0] }, () => {
                      if (this.state.currentAnimation != null &&
                        this.state.currentAnimation.frames.length > 0) {
                        this.setState({ currentFrame: this.state.currentAnimation.frames[0] })
                      }
                    })
                  }
                })
              }
            })
        }
      })
    })
  }

  render() {
    return (
      <div className="home grid">
        <SelectableList title="Collections" items={this.state.collectionNames} />
        <SelectableList title="Animations" items={this.state.currentCollection?.animations.map(anim => anim.name) as string[]} />
        <SelectableList title="Frames" items={this.state.currentAnimation?.frames as string[]} />
      </div>
    )
  }

  debug(msg: string) {
    console.log("Debug: " + msg)
    invoke('debug', { msg: msg })
  }
}