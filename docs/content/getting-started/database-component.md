---
title: Database
---

Connect and talk to a database
===

Getting a database hooked up and making calls to the database is an extremely simple process in Wick. Let's go through it:

In order to connect and talk to a database, we will be creating a Wick component. You can start with a new `.wick` file. Let's call our file `db.wick`.

```yaml
name: demo_db
kind: wick/component@v1
```

After naming and declaring the [`kind`]( {{< ref "configuration/reference/v1#componentconfiguration" >}}) of the component, we can get hooked up to our database (you will need your own database up and running).

We declare our database as a [`resource`]( {{< ref "configuration/reference/v1#componentconfiguration" >}}). Resources are what a component relies on to execute its operations.

```yaml
resources:
  - name: MYDATABASE
    resource:
      kind: wick/resource/url@v1
      url: postgres://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME
```

All we need to do is name [our resource]( {{< ref "configuration/reference/v1#resourcebinding" >}}) so we can reference it in other places in our app, declare its `kind`, and provide the database URL.

The last thing we need is to finish declaring our [sql component]( {{< ref "configuration/reference/v1#sqlcomponent" >}}) and assign to it the database resource.

```yaml
component:
  kind: wick/component/sql@v1
  resource: MYDATABASE
  tls: false
```

And that's all you need to hook up to a database!

### Making database calls

Now let's see how to make calls to the database.

We will be defining these queries in our [`operations`]( {{< ref "configuration/reference/v1#sqloperationdefinition" >}}) section of the YAML.

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

After naming the operation, each one consists of four fields: inputs, outputs, arguments, and the query.

***inputs + outputs:*** We named and declared the type of input and output. The `type: object` serves as our `any` type for our output.

***arguments:*** Sets up the sequence of your inputs. The order of the arguments here will be the order of the inputs in your query.

***query:*** Insert the database query here. (Note: You would replace the values with $1, $2, $3, etc. based on the order of your arguments.)

Hooking up and setting up calls to a database is as simple as that in Wick! You can follow the same `operations` structure to add as many unique calls to the database as you'd like, each with their own inputs and outputs.

### Invoking

Lastly, let's just go over how to run any of our operations we create.

It would be a simple `invoke` call on the command line following this structure:

```
wick invoke <file name> <operation name> -- --args
```

You can use `wick invoke --trace` instead for debugging purposes.

Our command line prompt for the file we just made would be:

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

And just like that, we've got a database hooked up and are making calls to it. You can use this same structure to hook up to any database and make as many calls as you'd like. To see more database examples, check out our [examples repo](https://github.com/candlecorp/wick/tree/main/examples/db).

Note: Wick now also has the `wick new component sql` command that will help you get started with a database component. It will create a new `.wick` file with the database resource and a sample operation.
