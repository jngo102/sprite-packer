import { Component } from 'react'
import { Divider, Grid, List, ListItem, ListItemButton, ListItemText, ListSubheader } from '@mui/material'

interface SelectableListProps {
    items: string[]
    onSelectItem: any
    selectedItem: string
    title: string
}

interface SelectableListState {
    onSelectItem: any
}

export default class SelectableList extends Component<SelectableListProps, SelectableListState> {
    constructor(props: SelectableListProps) {
        super(props)
        this.state = {
            onSelectItem: props.onSelectItem
        }
    }

    render() {
        return (
            <Grid item>
                <List dense
                    disablePadding
                    sx={{
                        maxHeight: window.innerHeight - 64,
                        overflowY: "auto"
                    }}>
                    <ListSubheader>
                        {this.props.title}
                        <Divider />
                    </ListSubheader>
                    {this.props.items?.map(item => (
                        <ListItem disablePadding key={item}>
                            <ListItemButton
                                onClick={_ => this.updateSelected(item)}
                                selected={this.props.selectedItem == item}>
                                <ListItemText primary={item} />
                            </ListItemButton>
                        </ListItem>
                    ))}
                </List>
            </Grid>
        )
    }

    updateSelected(item: string) {
        if (this.props.onSelectItem != null) {
            this.props.onSelectItem(item)
        }
    }
}