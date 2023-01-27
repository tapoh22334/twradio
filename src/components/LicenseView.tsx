import './LicenseView.css';

import * as React from "react";
import Typography from '@mui/material/Typography';
import Box from '@mui/material/Box';

export const Licenses = () => {
    const [textAbout, setTextAbout] = React.useState("");
    const [textYarn, setTextYarn] = React.useState("");
    const [textCargo, setTextCargo] = React.useState("");

    const LicensesAbout = require('./resource/ABOUT.txt');
    const LicensesYarn = require('./resource/THIRD-PARTY-NOTICES-yarn.txt');
    const LicensesCargo =  require('./resource/THIRD-PARTY-NOTICES-cargo.txt');

    fetch(LicensesAbout)
        .then((response) => response.text())
        .then((textContent) => {
                setTextAbout(textContent);
                });

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

        <Box className="License">
            <div
                dangerouslySetInnerHTML={{ __html: textAbout }}
            />
            <h2>Licenses</h2>
            <div
                dangerouslySetInnerHTML={{ __html: textCargo }}
            />
            <Box>
                <pre >{textYarn}</pre>
            </Box>
        </Box>
    );
}

export default Licenses;
