const express = require('express');
const cors = require('cors');
const axios = require('axios');
require('dotenv').config();

const app = express();

app.use(cors());
app.use(express.json());

app.get('/', (req, res) => {
    res.send('Backend server is running');
});

// Moved getAccessToken function outside of route handlers
async function getAccessToken() {
    const auth = Buffer.from(`${process.env.CONSUMER_KEY}:${process.env.CONSUMER_SECRET}`).toString('base64');

    try {
        const response = await axios.get('https://sandbox.safaricom.co.ke/oauth/v1/generate?grant_type=client_credentials', {
            headers: {
                Authorization: `Basic ${auth}`
            }
        });
        return response.data.access_token;
    } catch (error) {
        console.error('Error getting access token:', error.response?.data || error.message);
        throw new Error('Failed to get access token');
    }
}

// Single /pay endpoint (removed duplicate)
app.post('/pay', async (req, res) => {
    const { phone, amount, item } = req.body;
    
    // Input validation
    if (!phone || !amount) {
        return res.status(400).json({ error: 'Phone and amount are required' });
    }

    // Validate phone number format
    if (!phone.startsWith('254') || phone.length !== 12) {
        return res.status(400).json({ error: 'Invalid phone number format. Use 254XXXXXXXXX' });
    }

    try {
        const aToken = await getAccessToken();
        const timestamp = new Date().toISOString().replace(/[-:TZ.]/g, '').slice(0, 14);
        const password = Buffer.from(`${process.env.SHORTCODE}${process.env.PASSKEY}${timestamp}`).toString('base64');

        const stkPushData = {
            BusinessShortCode: process.env.SHORTCODE,
            Password: password, 
            Timestamp: timestamp,
            TransactionType: 'CustomerPayBillOnline',
            Amount: amount,
            PartyA: phone,
            PartyB: process.env.SHORTCODE,
            PhoneNumber: phone,
            CallBackURL: process.env.CALLBACK_URL,
            AccountReference: item || 'TestPayment',
            TransactionDesc: 'Website Payment'
        };

        const stkresponse = await axios.post(
            'https://sandbox.safaricom.co.ke/mpesa/stkpush/v1/processrequest', 
            stkPushData, 
            { 
                headers: { 
                    Authorization: `Bearer ${aToken}`,
                    'Content-Type': 'application/json'
                } 
            }
        );
        
        res.json({
            message: `M-Pesa STK Push sent to ${phone} for KES ${amount}`,
            data: stkresponse.data
        });
    } catch (error) {
        console.error('Error processing payment:', error.response ? error.response.data : error.message);
        res.status(500).json({ 
            error: 'Failed to process payment',
            details: error.response ? error.response.data : error.message
        });
    }
});

app.post('/callback', (req, res) => {
    console.log('Received M-Pesa callback:', req.body);
    // Process the callback - update payment status in database
    res.status(200).json({ ResultCode: 0, ResultDesc: "Success" });
});

const PORT = process.env.PORT || 5500;
app.listen(PORT, () => console.log(`Backend running on http://localhost:${PORT}`));