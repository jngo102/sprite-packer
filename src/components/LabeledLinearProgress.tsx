import { Component } from 'react'
import { Box, Grid, LinearProgress, LinearProgressProps, Typography } from '@mui/material'

interface LabeledLinearProgressProps extends LinearProgressProps {
    value: number
}

export default class LabeledLinearProgress extends Component<LabeledLinearProgressProps> {
    constructor(props: LabeledLinearProgressProps) {
        super(props)
    }

    render() {
        return (
            <Grid item xs={12}>
                <Box sx={{ display: 'flex', alignItems: 'center' }}>
                    <Box sx={{ width: '100%', mr: 1 }}>
                        <LinearProgress variant="determinate" color={this.props.value >= 100 ? "success" : "secondary"} {...this.props} />
                    </Box>
                    <Box sx={{ minWidth: 35 }}>
                        <Typography variant="body2" color="text.secondary">
                            {`${Math.round(this.props.value)}%`}
                        </Typography>
                    </Box>
                </Box>
            </Grid>
        )
    }
}