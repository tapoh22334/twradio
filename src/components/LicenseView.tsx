import './LicenseView.css';

import * as React from "react";
import Typography from '@mui/material/Typography';
import Box from '@mui/material/Box';

export const Licenses = () => {
    const [textYarn, setTextYarn] = React.useState("");
    const [textCargo, setTextCargo] = React.useState("");

    const LicensesYarn = require('./resource/THIRD-PARTY-NOTICES-yarn.txt');
    const LicensesCargo =  require('./resource/THIRD-PARTY-NOTICES-cargo.txt');

    fetch(LicensesYarn)
        .then((response) => response.text())
        .then((textContent) => {
                setTextYarn(textContent);
                });

    fetch(LicensesCargo)
        .then((response) => response.text())
        .then((textContent) => {
                setTextCargo(textContent);
                });

    return (

        <Box>
            <Typography variant="h3" gutterBottom>
                Licenses
            </Typography>
            <div
                dangerouslySetInnerHTML={{ __html: textCargo }}
            />
            <Box className="License">
                <pre >{textYarn}</pre>
            </Box>
        </Box>
    );
}

export default Licenses;
