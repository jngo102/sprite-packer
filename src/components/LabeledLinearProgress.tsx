import { Component } from 'react'
import { Box, Grid, LinearProgress, LinearProgressProps, Typography } from '@mui/material'

interface LabeledLinearProgressProps extends LinearProgressProps {
    text: string
    value: number
}

export default class LabeledLinearProgress extends Component<LabeledLinearProgressProps> {
    constructor(props: LabeledLinearProgressProps) {
        super(props)
    }

    render() {
        return (
            <Grid item xs={10}>
                <Box sx={{ display: this.props.hidden ? "none" : "flex" }}>
                    <Box sx={{ width: "100%" }}>
                        <LinearProgress variant="determinate" color={this.props.value >= 100 ? "success" : "secondary"} {...this.props} />
                    </Box>
                    <Box sx={{ minWidth: 64 }}>
                        <Typography variant="body2" color="text.secondary">
                            {`${this.props.text}: ${Math.round(this.props.value)}%`}
                        </Typography>
                    </Box>
                </Box>
            </Grid>
        )
    }
}