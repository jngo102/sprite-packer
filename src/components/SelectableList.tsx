import { Component } from 'react'
import { ListBox } from 'primereact/listbox'
import './SelectableList.css'

interface SelectableListProps {
    title: string
    items: string[]
}

interface SelectableListState {
    selectedItem: string | undefined
}

export default class SelectableList extends Component<SelectableListProps, SelectableListState> {
    constructor(props: SelectableListProps) {
        super(props)
        this.state = {
            selectedItem: props.items?.at(0),
        }
    }

    render() {
        return (
            <div className="card">
                <ListBox filter
                    filterPlaceholder={`Filter ${this.props.title}`}
                    listStyle={{ height: '256px', overflowY: 'auto' }}
                    onChange={(e) => this.setState({ selectedItem: e.value })}
                    options={this.props.items}
                    value={this.state.selectedItem}
                />
            </div>
        )
    }
}