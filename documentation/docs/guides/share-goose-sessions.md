---
sidebar_position: 5
title: Making Shared Goose Agents
sidebar_label: Share Sessions
---
import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';


A shared session in Goose is like a collaborative workspace where multiple people can work with the AI assistant together in real-time. Think of it similar to a shared Google Doc, but for AI assistance.

## Create a shared Agent
<Tabs>
    <TabItem value="cli" label="Goose CLI" default>
        The Goose CLI can generate a "recipe" which encapsulates all of the components to be shared with another Goose user. To share an agent, first create a recipe file.

        In your terminal with a Goose session running, input the following:
        ```sh
        ( O)> /recipe 
        ```
        Goose generates `recipe.yaml` and saves it for you as shown in the following output:

        ```sh
        Generating Recipe
        Saved recipe to .../block/goose/recipe.yaml
        ```
        Send the file to the Goose CLI user that wants to start a shared session.  For more information about the `Recipe` prompt completion command, see [Recipe](/docs/guides/goose-cli-commands#recipe)
    </TabItem>
    <TabItem value="ui" label="Goose Desktop">

        To share a session at any time, click the three dots in the top-right corner of the application and select **Make Agent from this session** from the dropdown menu. 

        Goose gathers the components to be shared and opens a dialog that provides an Agent URL to be provided to the Goose user who will share your session. In the following example, the user created an agent and will review the activities to be shared and remove them as needed. After the right set of activites is in the list, they send the URL to another Goose desktop user.

        ![Start a shared Agent](../assets/guides/shared-agent-created-ui.png)



    </TabItem>
</Tabs>

## Start the shared Agent 

<Tabs>
    <TabItem value="cli" label="Goose CLI" default>
        When another Goose CLI user sends you a recipe file, save it in the directory where you want to start your shared Agent.
        From your terminal, navigate to the directory where you saved the file, and run:
        ```sh
        goose run --<RECIPE_FILE_NAME> 
        ```
        The Goose CLI loads the recipe and creates a session with all of the components shared in the original session. 

    </TabItem>
    <TabItem value="ui" label="Goose Desktop">
        When another Goose desktop user wants to share an Agent, they send you an URL which you use to start the new shared Agent.

        Open a new tab on your browser and paste the shared Agent URL into the address bar and press the **enter** key on your keyboard. The browser requests your permission to start a new Goose session with the shared components. 

        The new Goose session shows a set of actions that you can take as clickable buttons. As shown in the following image, the shared Agent is shown as an action that summarizes the purpose of the shared Agent. To activate the agent, review the summary and then select the action.  

        ![Start a shared Agent](../assets/guides/start-shared-agent-ui.png)


    </TabItem>
</Tabs>



## What gets shared?
You may have started a project in a Goose session that needed information that your collaborating team member needs in the shared agent that you create. At the same time, you may have needed to give Goose information that you'd rather not share. 
### Shared components
The shared agent includes these components:
* Conversation history (all messages)
* Tool outputs and results
* Files or content created during the session
* Active extensions and their configurations
* Project context (when working within a project)

### Private components
The following components are not included in a shared agent:
* Global memories (stored in `~/.config/goose/memory`)
* Local memories (stored in .goose/memory)
* Personal API keys or credentials
* System-level configurations



## Common use cases
There are many reasons why you might want to create a shared agent. The following shared agent use  cases are just a starting point.

### Team Collaboration
* Working together on coding projects
* Troubleshooting technical issues
* Brainstorming sessions
* Training & Onboarding

### Teaching new team members
* Demonstrating workflows
* Sharing best practices
* Pair Programming

### Real-time code collaboration
* Code reviews
* Debugging sessions





