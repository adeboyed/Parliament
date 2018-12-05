# Communication spec
The communication between all of the nodes in Parliment is using gzipped Protocol Buffers. 

We are compressing the communication using gzip as it is a very fast compression scheme & it means less time spent transmitting the data.
Protocol buffers were chosen because they are language agonistic and a structured way of sending data over the network.

## Starting connection. user library -> cluster
**connection.proto**

a. When the user library wants to connect to the processing cluster, it will send a connection request.
`ConnectionRequest`

b. The cluster will reply with this message. The cluster may reject the connection request from the user, this could be for authentication or capacity reasons.
`ConnectionResponse`

## Sending a job to the cluster. user library -> cluster
**job.proto**

