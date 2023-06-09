---
title: Database
---

Connect and talk to a database
===

Getting a database hooked up and making calls to the database is an extremely simple process in Wick. Lets go through it:

In order to connect and talk to a database, we will be creating a Wick component. You can start with a new `.wick` file- lets call our file `db.wick`.

```yaml
name: demo_db
kind: wick/component@v1
```
After naming and declaring the `kind` of component, we can get hooked up to our database. (You will need your own database up and running.)

We declare our database as a resource. Resources are what a component relies on to excute its operations.

```yaml
resources:
  - name: MYDATABASE
    resource:
      kind: wick/resource/url@v1
      url: postgres://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME
```

All we need to do is name our resource so we can reference it in other places in our app, declare its `kind`, and provide the database url.

The last thing we need is to finish declaring our component and assign to it, the database resource.

```yaml
component:
  kind: wick/component/sql@v1
  resource: MYDATABASE
  tls: false
```

And thats all you need to hook up to a database!

### Making database calls

Now lets see how to make calls to the database.

We will be defining these queries in our `operations` section of the yaml.

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

After naming the operation, each one consists of 4 fields. The inputs, outputs, arguments, and the query.

***inputs + outputs:*** We named and declared the type of input and output. The `type: object` serves as our `any` type for our output.

***arguments:*** Sets up the sequence of your inputs. The order of the arguments here will be the order of the inputs in your query.

***query:*** insert database query here. (Note: You would replace the values with $1, $2, $3... etc. based on the order of your arguments.)

Hooking up and setting up calls to a database is as simple as that in wick! You can follow the same `operations` structure to add as many unique calls to the database as you'd like, each with their own inputs and outputs.

### Invoking

Lastly, lets just go over how to run any of our operations we create.

It would be a simple `invoke` call on the command line following this structure:

```
wick invoke <file name> <operation name> -- --args
```

You can use `wick invoke --trace` instead for debugging purposes.

Our command line prompt for the file we just made would be:
```
wick invoke db.wick get_user -- --id=1
```
This would return a json object conatining all the relevant data for user id 1.

### Done!

And just like that, we've got a database hooked up and are making calls to it. You can use this same structure to hook up to any database and make as many calls you'd like.
