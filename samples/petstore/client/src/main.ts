import newClient from "openapi-fetch";
import type { paths, components } from "../openapi";

async function main() {
    const newPet: components["schemas"]["CreatePetRequest"] = (() => {
        let [, , petName] = process.argv;
        if (petName) {
            return {
                petName,
                tag: "user"
            }
        } else {
            const now = new Date();
            petName = `pet${now.getHours()}${now.getMinutes()}${now.getSeconds()}`;

            console.warn(`System generated a pet's name: "${petName}"`);
            console.warn(`You can specify one via a command line argument.`);

            return {
                petName,
                tag: "system"
            }
        }
    })();

    const client = newClient<paths>({ baseUrl: "http://localhost:5050" });

    {
        const { data, error } = await client.GET("/pets");
        if (error) {
            console.log(`error from "GET /pets": %o`, error);
            return;
        }
        console.log(`data from "GET /pets": %o`, data);
    }

    {
        const { data, error } = await client.POST("/pets", {
            body: newPet
        });
        if (error) {
            console.log(`error from "POST /pets": %o`, error);
            return;
        }
        console.log(`data from "POST /pets": %o`, data);
    }
    
    {
        const { data, error } = await client.GET("/pets");
        if (error) {
            console.log(`error from "GET /pets": %o`, error);
            return;
        }
        console.log(`data from "GET /pets": %o`, data);
    }
}

await main();
