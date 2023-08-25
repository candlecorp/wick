---
title: Database
weight: 3
---

# Connect and Talk to a Database

Getting a database connected and making calls to the database is an extremely simple process in Wick. Let's go through it:

To connect and talk to a database, we need to create a Wick component. Start with a new `.wick` file and name it `db.wick`.

```yaml
name: demo_db
kind: wick/component@v1
```

After naming and declaring the {{<v1ref "componentconfiguration">}}kind{{</v1ref>}} of the component, we can establish a connection to our database (ensure that your own database is up and running).

We declare our database as a {{<v1ref "componentconfiguration">}}resource{{</v1ref>}}. Resources are what a component relies on to execute its operations.

```yaml
resources:
  - name: MYDATABASE
    resource:
      kind: wick/resource/url@v1
      url: postgres://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME
```

In this section, we bind {{<v1ref "resourcebinding">}}our resource{{</v1ref>}}, declare its `kind`, and provide the database URL.

The next step is to complete the declaration of our {{<v1ref "sqlcomponent">}}SQL component{{</v1ref>}} and assign the database resource to it.

```yaml
component:
  kind: wick/component/sql@v1
  resource: MYDATABASE
  tls: false
```

And that's all you need to do to connect to a database!

### Making Database Calls

Now, let's see how to make calls to the database.

We define these queries in the {{<v1ref "sqloperationdefinition">}}operations{{</v1ref>}} section of the YAML.

```yaml
component:
  kind: wick/component/sql@v1
  resource: MYDATABASE
  tls: false
  operations:
   - name: get_user
      inputs:
        - name: id
          type: i32
      outputs:
        - name: output
          type: object
      arguments:
        - id
      query: |
        SELECT * FROM users WHERE id = $1
```

For each operation, we name it and provide details such as inputs, outputs, arguments, and the query.

**_inputs + outputs:_** We name and declare the type of input and output. The `type: object` serves as the `any` type for the output.

**_arguments:_** Sets up the sequence of inputs. The order of the arguments here will match the order of the inputs in your query.

**_query:_** Insert the database query here. (Note: Replace the values with $1, $2, $3, etc. based on the order of your arguments.)

Connecting to a database and making calls to it is as simple as that in Wick! You can use the same `operations` structure to add as many unique calls to the database as you like, each with their own inputs and outputs.

### Invoking

Lastly, let's go over how to run any of the operations we created.

You can use the `invoke` command on the command line with the following structure:

```
wick invoke <file name> <operation name> -- --args
```

For debugging purposes, you can use `wick invoke --trace`.

For the file we just created, the command line prompt would be:

```
wick invoke db.wick get_user -- --id=1
```

This would return a JSON object containing all the relevant data for user id 1.

### Done!

Here is what the complete `db.wick` file would look like:

```yaml
name: demo_db
kind: wick/component@v1
resources:
  - name: MYDATABASE
    resource:
      kind: wick/resource/url@v1
      url: postgres://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME
component:
  kind: wick/component/sql@v1
  resource: MYDATABASE
  tls: false
  operations:
   - name: get_user
      inputs:
        - name: id
          type: i32
      outputs:
        - name: output
          type: object
      arguments:
        - id
      query: |
        SELECT * FROM users WHERE id = $1
```

And just like that, we've connected to a database and are making calls to it. You can use this same structure to connect to any database and make as many calls as you need. For more database examples, check out our [examples repository](https://github.com/candlecorp/wick/tree/main/examples/db).

Note: Wick now also includes the `wick new component sql` command, which helps you get started with a database component. It creates a new .wick file with the database resource and a sample operation.
