import axios from "axios";

export class ApiSender {
    private endpoint: string;

    constructor(endpoint: string) {
        this.endpoint = endpoint;
    }

    public async send(data: object): Promise<void> {
        try {
            const response = await axios.post(this.endpoint, data);
            console.log("Data successfully sent:", response.data);
        } catch (error) {
            if (error instanceof Error) {
                console.error("Failed to send data:", error.message);
            } else {
                console.error("An unknown error occurred while sending data:", error);
            }
        }
    }
}
